require("dotenv").config();
const solanaWeb3 = require("@solana/web3.js");
const crypto = require("crypto");
const keccak256 = require("keccak256");
const secp256k1 = require("secp256k1");
const eth_utils = require("ethereumjs-util");
const borsh = require("borsh");

let SIGNER_GROUP_SIZE = 33;
let VALID_SIGNER_SIZE = 53;
let AUDIUS_PROGRAM = new solanaWeb3.PublicKey(
  "Eed4UXg3TqTkzbgT47WWMQtbzpxMUeHitt2jfAa6pmep"
);
let CREATE_AND_VERIFY_PROGRAM = new solanaWeb3.PublicKey(
  "Fs2Fmuo266ohhQjGRC6FCufjYaCHkuhyyRi3zn2xqrQP"
);
let INSTRUCTIONS_PROGRAM = new solanaWeb3.PublicKey(
  "Sysvar1nstructions1111111111111111111111111"
);

let feePayer = new solanaWeb3.Account([252,1,35,131,28,114,106,11,143,29,15,86,81,148,58,2,176,19,127,110,76,255,249,56,140,236,31,209,51,176,103,166,231,243,24,228,226,124,136,74,78,251,163,47,230,6,142,27,156,140,246,92,108,114,163,237,226,243,170,124,76,24,62,125,
]);
let owner = new solanaWeb3.Account([63,181,8,61,246,121,106,102,159,113,145,62,38,196,23,242,102,18,191,255,46,250,34,47,102,160,157,129,21,233,209,194,32,76,67,148,133,69,126,66,181,10,4,130,39,21,204,15,97,166,77,87,142,255,146,170,86,42,173,154,120,29,56,211,
]);

class Assignable {
  constructor(data) {
    Object.assign(this, data);
  }
}

class TrackData extends Assignable {}
class InstructionArgs extends Assignable {}
class InstructionEnum extends Assignable {}
class SignatureData extends Assignable {}
class InitSignerGroup extends Assignable {}
class InitValidSigner extends Assignable {}
class ClearValidSigner extends Assignable {}

let url = solanaWeb3.clusterApiUrl("devnet", false);

let devnetConnection = new solanaWeb3.Connection(url);

async function validateSignature(validSigner, privateKey, message) {
  let privKey = Buffer.from(privateKey, "hex");
  let pubKey = secp256k1.publicKeyCreate(privKey, false);

  let validSignerPubK = new solanaWeb3.PublicKey(validSigner);
  let accInfo = await devnetConnection.getAccountInfo(validSignerPubK);
  let signerGroup = new solanaWeb3.PublicKey(
    accInfo.data.toJSON().data.slice(1, 33)
  ); // cut off version and eth address from valid signer data

  let msg = Buffer.from(message).toJSON().data;

  let msg_hash = keccak256(msg);

  const sigObj = secp256k1.ecdsaSign(Uint8Array.from(msg_hash), privKey);

  let transaction = new solanaWeb3.Transaction();
  let instructionSchema = new Map([
    [
      InstructionEnum,
      {
        kind: "enum",
        field: "choose",
        values: [["initSignerGroup", InitSignerGroup], ["initValidSigner", InitValidSigner], ["clearValidSigner", ClearValidSigner], ["instruction", SignatureData]],
      },
    ],
    [
      SignatureData,
      {
        kind: "struct",
        fields: [
          ["signature", [64]],
          ["recovery_id", "u8"],
          ["message", [msg.length]],
        ],
      },
    ],
  ]);

  let instructionArgs = new SignatureData({
    signature: Array.from(sigObj.signature),
    recovery_id: sigObj.recid,
    message: msg,
  });

  let instructionData = new InstructionEnum({
    instruction: instructionArgs,
    choose: "instruction",
  });

  let serializedInstructionArgs = borsh.serialize(
    instructionSchema,
    instructionData
  );

  var lenBuffer = new ArrayBuffer(4);  // size of len data in Rust 4 bytes
  if (msg.length < 256) {
    new DataView(lenBuffer).setInt8(0, msg.length);
  } else {
    new DataView(lenBuffer).setInt16(0, msg.length);
  }

  let serializedInstructionArray = serializedInstructionArgs.toJSON().data;
  serializedInstructionArray.splice(66, 0, ...new Uint8Array(lenBuffer));

  let secpInstruction = solanaWeb3.Secp256k1Program.createInstructionWithPublicKey(
    {
      publicKey: pubKey,
      message: msg,
      signature: sigObj.signature,
      recoveryId: sigObj.recid,
    }
  );

  transaction.add(secpInstruction);

  transaction.add({
    keys: [
      { pubkey: validSignerPubK, isSigner: false, isWritable: false },
      { pubkey: signerGroup, isSigner: false, isWritable: false },
      { pubkey: INSTRUCTIONS_PROGRAM, isSigner: false, isWritable: false },
    ],
    programId: AUDIUS_PROGRAM,
    data: serializedInstructionArray,
  });

  let signature = await solanaWeb3.sendAndConfirmTransaction(
    devnetConnection,
    transaction,
    [feePayer]
  );

  console.log("Signature: ", signature);
}

async function createAndVerifyMessage(
  validSigner,
  privateKey,
  userId,
  trackId,
  source
) {
  let privKey = Buffer.from(privateKey, "hex");
  let pubKey = secp256k1.publicKeyCreate(privKey, false);

  let validSignerPubK = new solanaWeb3.PublicKey(validSigner);
  let accInfo = await devnetConnection.getAccountInfo(validSignerPubK);
  let signerGroup = new solanaWeb3.PublicKey(
    accInfo.data.toJSON().data.slice(1, 33)
  ); // cut off version and eth address from valid signer data

  let trackData = new TrackData({
    user_id: userId,
    track_id: trackId,
    source: source,
  });
  let trackDataSchema = new Map([
    [
      TrackData,
      {
        kind: "struct",
        fields: [
          ["user_id", "string"],
          ["track_id", "string"],
          ["source", "string"],
        ],
      },
    ],
  ]);

  const serializedTrackData = borsh.serialize(trackDataSchema, trackData);
  let msg_hash = keccak256(serializedTrackData.toJSON().data);

  const sigObj = secp256k1.ecdsaSign(Uint8Array.from(msg_hash), privKey);

  let instructionSchema = new Map([
    [
      InstructionEnum,
      {
        kind: "enum",
        field: "choose",
        values: [["instruction", InstructionArgs]],
      },
    ],
    [
      InstructionArgs,
      {
        kind: "struct",
        fields: [
          ["track_data", TrackData],
          ["signature", [64]],
          ["recovery_id", "u8"],
        ],
      },
    ],
    [
      TrackData,
      {
        kind: "struct",
        fields: [
          ["user_id", "string"],
          ["track_id", "string"],
          ["source", "string"],
        ],
      },
    ],
  ]);

  let instructionArgs = new InstructionArgs({
    track_data: trackData,
    signature: Array.from(sigObj.signature),
    recovery_id: sigObj.recid,
  });

  let instructionData = new InstructionEnum({
    instruction: instructionArgs,
    choose: "instruction",
  });

  const serializedInstructionArgs = borsh.serialize(
    instructionSchema,
    instructionData
  );

  let transaction = new solanaWeb3.Transaction();

  let secpInstruction = solanaWeb3.Secp256k1Program.createInstructionWithPublicKey(
    {
      publicKey: pubKey,
      message: serializedTrackData.toJSON().data,
      signature: sigObj.signature,
      recoveryId: sigObj.recid,
    }
  );

  transaction.add(secpInstruction);

  transaction.add({
    keys: [
      { pubkey: validSignerPubK, isSigner: false, isWritable: false },
      { pubkey: signerGroup, isSigner: false, isWritable: false },
      { pubkey: AUDIUS_PROGRAM, isSigner: false, isWritable: false },
      { pubkey: INSTRUCTIONS_PROGRAM, isSigner: false, isWritable: false },
    ],
    programId: CREATE_AND_VERIFY_PROGRAM,
    data: serializedInstructionArgs,
  });

  let signature = await solanaWeb3.sendAndConfirmTransaction(
    devnetConnection,
    transaction,
    [feePayer]
  );

  console.log("Signature: ", signature);
}
exports.validateSignature = validateSignature;
exports.createAndVerifyMessage = createAndVerifyMessage;
