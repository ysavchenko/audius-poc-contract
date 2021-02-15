#!/usr/bin/env node

const commander = require('commander');
const instr = require("./audius_instructions");

commander
    .version("1.0.0")
    .description("Audius solana program CLI.");

commander.command("create-signer-group")
    .description("Create new signer group")
    .action(() => {
        instr.createSignerGroup().then(() => {
            console.log("New signer group was created successfully")
        });
    });

commander.command("create-valid-signer <signerGroup>")
    .description("Create new valid signer with Secp256k1 private key")
    .action((signerGroup) => {
        instr.createValidSigner(signerGroup).then(() => {
            console.log("New valid signer was created and added to pointed signer group");
        })
    })

commander.command("send-message <validSigner> <privateKey> <message>")
    .description("Sign message and send signature to the program to verify it")
    .action((validSigner, privateKey, message) => {
        instr.validateSignature(validSigner, privateKey, message).then(() => {
            console.log("Signature was verified successfully and message sent");
        })
    })

commander.parse(process.argv);