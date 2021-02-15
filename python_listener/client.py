from time import sleep

import base58
from solana.rpc.api import Client

AUDIUS_PROGRAM = "3QqhXLvBgPZ4DCV3YjyzpiQWfeR4Lf2bSKqSnj5c8wkE"
SECP_PROGRAM = "KeccakSecp256k11111111111111111111111111111"

SLEEP_TIME = 3

SOLANA_ENDPOINT = "https://devnet.solana.com"

http_client = Client(SOLANA_ENDPOINT)

slot_from = None

print(f"Listening for transactions on {AUDIUS_PROGRAM}")

while True:
    if not slot_from:
        slot_from = http_client.get_slot()['result']

    transaction = http_client.get_confirmed_signature_for_address2(AUDIUS_PROGRAM, limit=1)

    if transaction['result'][0]['slot'] > slot_from:
        slot_from = transaction['result'][0]['slot']
        tx_info = http_client.get_confirmed_transaction(transaction['result'][0]['signature'])
        if SECP_PROGRAM in tx_info['result']['transaction']['message']['accountKeys']:
            audius_program_index = tx_info['result']['transaction']['message']['accountKeys'].index(AUDIUS_PROGRAM)
            for instruction in tx_info['result']['transaction']['message']['instructions']:
                if instruction['programIdIndex'] == audius_program_index:
                    signed_msg = base58.b58decode(instruction['data'])[65:].decode()
                    print(f"TX: {tx_info['result']['transaction']['signatures'][0]}")
                    print(f"Message: \"{signed_msg}\"")
    sleep(SLEEP_TIME)
    print("...")
