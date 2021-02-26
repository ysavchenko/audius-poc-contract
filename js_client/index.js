#!/usr/bin/env node

const commander = require('commander');
const instr = require("./audius_instructions");

commander
    .version("1.0.0")
    .description("Audius solana program CLI.");

commander.command("send-message <validSigner> <privateKey> <message>")
    .description("Sign message and send signature to the program to verify it")
    .action((validSigner, privateKey, message) => {
        instr.validateSignature(validSigner, privateKey, message).then(() => {
            console.log("Signature was verified successfully and message sent");
        })
    })

commander.parse(process.argv);