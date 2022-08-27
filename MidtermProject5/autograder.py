import os
import time
import requests
import json
import subprocess
import sys

def is_blockchain_valid(json1, json2, json3):
    print('>>> Start blockchain test... <<<')

    min_len = min(len(json1), len(json2), len(json3))
    max_len = max(len(json1), len(json2), len(json3))
    print("min_len", min_len)
    # test 1 for longest chain length
    if min_len >= 50:
        print('\033[32m' + '[Test 1] - Longest Chain Case passed!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 1] - Longest Chain Case failed, minimal length is: ' + str(min_len) + '\033[0m')

    # test 2 for lengths diff
    if max_len - min_len <= 3:
        print('\033[32m' + '[Test 2] - Length Difference Case passed!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 2] - Length Difference Case failed, length diff is: ' + str(max_len-min_len) + '\033[0m')

    # test 3 for chains' blocks diff
    l = [json1[:min_len], json2[:min_len], json3[:min_len]]
    if all([len(set(i)) == 1 for i in zip(*l)]):
        print('\033[32m' + '[Test 3] - Common Prefix Case passed!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 3] - Common Prefix Case failed!' + '\033[0m')


def is_transaction_valid(tx1, tx2, tx3):
    print('>>> Start transaction test... <<<')

    blockNum1, blockNum2, blockNum3 = len(tx1), len(tx2), len(tx3)  # blockchain's length

    print(blockNum1, blockNum2, blockNum3)

    txNum1 = sum(len(tx1[i]) for i in range(blockNum1))             # transactions amount
    txNum2 = sum(len(tx2[i]) for i in range(blockNum2))
    txNum3 = sum(len(tx3[i]) for i in range(blockNum3))

    print(txNum1, txNum2, txNum3)

    # test 1 for tx throughput
    min_throughput = min(txNum1, txNum2, txNum3)
    if min_throughput >= 500:
        print('\033[32m' + '[Test 1] - Transaction Throughput Case passed!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 1] - Transaction Throughput Case failed, minimal throughput is: ' + str(min_throughput) + '\033[0m')

    # test 2 for tx per block
    avg1, avg2, avg3 = txNum1//(blockNum1-1), txNum2//(blockNum2-1), txNum3//(blockNum3-1)
    if 10<=min(avg1, avg2, avg3) and max(avg1, avg2, avg3)<=500:
        print('\033[32m' + '[Test 2] - Transaction Per Block Case passed!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 2] - Transaction Per Block Case failed, the min is: ' + str(min(avg1, avg2, avg3)) + ', the max is: ' + str(max(avg1, avg2, avg3)) + '\033[0m')

    # test 3 for duplicate
    set1, set2, set3 = set(), set(), set()
    for tx in tx1:
        set1.update(tx)
    for tx in tx2:
        set2.update(tx)
    for tx in tx3:
        set3.update(tx)
    portion = min(len(set1)/txNum1, len(set2)/txNum2, len(set3)/txNum3)
    if portion >= 0.9:
        print('\033[32m' + '[Test 3] - Transaction Duplication Case passed!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 3] - Transaction Duplication Case failed, duplication is: ' + str(portion) + '\033[0m')

    # test 4 for common prefix
    if tx1[1][0] == tx2[1][0] and tx1[1][0] == tx3[1][0]:
        print('\033[32m' + '[Test 4] - Transaction Common Prefix Case passed!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 4] - Transaction Common Prefix Case failed, three transactions are: ' + str(tx1[1][0]) + ', ' + str(tx2[1][0]) + ', ' + str(tx3[1][0]) + '\033[0m')

if __name__ == '__main__':
    try:
        # start processes
        print(os.getcwd())
        peer1 = subprocess.Popen('cargo run -- -vvv --p2p 127.0.0.1:6000 --api 127.0.0.1:7000', shell=True, cwd=os.getcwd(), start_new_session=True)
        peer2 = subprocess.Popen('cargo run -- -vvv --p2p 127.0.0.1:6001 --api 127.0.0.1:7001 -c 127.0.0.1:6000', shell=True, cwd=os.getcwd(), start_new_session=True)
        peer3 = subprocess.Popen('cargo run -- -vvv --p2p 127.0.0.1:6002 --api 127.0.0.1:7002 -c 127.0.0.1:6001', shell=True, cwd=os.getcwd(), start_new_session=True)
        time.sleep(10)

        print('[Peer] - Peer0 starts..')
        print('[Peer] - Peer1 starts..')
        print('[Peer] - Peer2 starts..')

        miner1 = subprocess.Popen('curl --request GET http://127.0.0.1:7000/miner/start?lambda=0', shell=True, cwd=os.getcwd(), start_new_session=True)
        miner2 = subprocess.Popen('curl --request GET http://127.0.0.1:7001/miner/start?lambda=0', shell=True, cwd=os.getcwd(), start_new_session=True)
        miner3 = subprocess.Popen('curl --request GET http://127.0.0.1:7002/miner/start?lambda=0', shell=True, cwd=os.getcwd(), start_new_session=True)

        print('[Peer] - Peer0 starts mining...')
        print('[Peer] - Peer1 starts mining...')
        print('[Peer] - Peer2 starts mining...')

        tx1 = subprocess.Popen('curl --request GET http://127.0.0.1:7000/tx-generator/start?theta=100', shell=True, cwd=os.getcwd(), start_new_session=True)
        tx2 = subprocess.Popen('curl --request GET http://127.0.0.1:7001/tx-generator/start?theta=100', shell=True, cwd=os.getcwd(), start_new_session=True)
        tx3 = subprocess.Popen('curl --request GET http://127.0.0.1:7002/tx-generator/start?theta=100', shell=True, cwd=os.getcwd(), start_new_session=True)

        print('[Peer] - Peer0 starts generate Tx...')
        print('[Peer] - Peer1 starts generate Tx...')
        print('[Peer] - Peer2 starts generate Tx...')

        # run for some time
        for remaining in range(300, 0, -1):
            sys.stdout.write("\r")
            sys.stdout.write("\033[92m" + "{:2d} seconds remaining.".format(remaining) + "\033[0m")
            sys.stdout.flush()
            time.sleep(1)

        # fetch data
        longest_chain1 = json.loads(str(requests.get('http://127.0.0.1:7000/blockchain/longest-chain').content.decode('utf-8')))
        longest_chain2 = json.loads(str(requests.get('http://127.0.0.1:7001/blockchain/longest-chain').content.decode('utf-8')))
        longest_chain3 = json.loads(str(requests.get('http://127.0.0.1:7002/blockchain/longest-chain').content.decode('utf-8')))

        longest_transaction1 = json.loads(str(requests.get('http://127.0.0.1:7000/blockchain/longest-chain-tx').content.decode('utf-8')))
        longest_transaction2 = json.loads(str(requests.get('http://127.0.0.1:7001/blockchain/longest-chain-tx').content.decode('utf-8')))
        longest_transaction3 = json.loads(str(requests.get('http://127.0.0.1:7002/blockchain/longest-chain-tx').content.decode('utf-8')))

        # begin test
        print('\n')
        is_blockchain_valid(longest_chain1, longest_chain2, longest_chain3)
        print('------------------------------------')
        is_transaction_valid(longest_transaction1, longest_transaction2, longest_transaction3)

    finally:
        peer1.kill()
        peer2.kill()
        peer3.kill()
        miner1.kill()
        miner2.kill()
        miner3.kill()
        tx1.kill()
        tx2.kill()
        tx3.kill()
        print('[Peer] - All peers killed!')

