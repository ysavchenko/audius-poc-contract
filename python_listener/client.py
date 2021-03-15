from time import sleep

import base58
import binascii
import codecs
from solana.rpc.api import Client

AUDIUS_PROGRAM = "Fm4g3bGuezevgqSHopjEDGRGCtVU6CDpFXZE3832EzGs"
SECP_PROGRAM = "KeccakSecp256k11111111111111111111111111111"

SLEEP_TIME = 3

# SOLANA_ENDPOINT = "https://devnet.solana.com"
SOLANA_ENDPOINT = "http://localhost:8899"

http_client = Client(SOLANA_ENDPOINT)

slot_from = None

while True:
    if not slot_from:
        slot_from = http_client.get_slot()["result"]

    transaction = http_client.get_confirmed_signature_for_address2(
        AUDIUS_PROGRAM, limit=1
    )

    if transaction["result"][0]["slot"] > slot_from:
        slot_from = transaction["result"][0]["slot"]
        tx_info = http_client.get_confirmed_transaction(
            transaction["result"][0]["signature"]
        )
        if SECP_PROGRAM in tx_info["result"]["transaction"]["message"]["accountKeys"]:
            audius_program_index = tx_info["result"]["transaction"]["message"][
                "accountKeys"
            ].index(AUDIUS_PROGRAM)
            for instruction in tx_info["result"]["transaction"]["message"][
                "instructions"
            ]:
                if instruction["programIdIndex"] == audius_program_index:
                    hex_data = binascii.hexlify(
                        bytearray(list(base58.b58decode(instruction["data"])))
                    )

                    l1 = int(hex_data[2:4], 16)
                    start_data1 = 10
                    end_data1 = l1 * 2 + start_data1

                    l2 = int(hex_data[end_data1 : end_data1 + 2], 16)
                    start_data2 = end_data1 + 8
                    end_data2 = l2 * 2 + start_data2

                    l3 = int(hex_data[end_data2 : end_data2 + 2], 16)
                    start_data3 = end_data2 + 8
                    end_data3 = l3 * 2 + start_data3

                    print(
                        f"Signed data:\nuser_id: {codecs.decode(hex_data[start_data1:end_data1], 'hex')}\ntrack_id: {codecs.decode(hex_data[start_data2:end_data2], 'hex')}\nsource: {codecs.decode(hex_data[start_data3:end_data3], 'hex')}"
                    )

                    print(
                        f"Get 'send message' transaction: {tx_info['result']['transaction']['signatures'][0]}"
                    )
    sleep(SLEEP_TIME)
