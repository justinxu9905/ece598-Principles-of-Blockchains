import os
from pydoc import plain
import time
import requests
import json
import subprocess
import sys

def is_blockchain_valid(json1, json2, json3):
    print('>>> Start blockchain test... <<<')

    min_len = min(len(json1), len(json2), len(json3))
    max_len = max(len(json1), len(json2), len(json3))

    # test 1 for longest chain length
    if min_len >= 20:
        print('\033[32m' + f'[Test 0] - Longest Chain Case passed! max_len: {max_len}, min_len: {min_len}' + '\033[0m')
    else:
        print('\033[31m' + '[Test 0] - Longest Chain Case failed, minimal length is: ' + str(min_len) + '\033[0m')

    # test 2 for lengths diff
    if max_len - min_len <= 3:
        print('\033[32m' + '[Test 0] - Length Difference Case passed!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 0] - Length Difference Case failed, length diff is: ' + str(max_len-min_len) + '\033[0m')

    # test 3 for chains' blocks diff
    l = [json1[:min_len], json2[:min_len], json3[:min_len]]
    if all([len(set(i)) == 1 for i in zip(*l)]):
        print('\033[32m' + '[Test 0] - Common Prefix Case passed!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 0] - Common Prefix Case failed!' + '\033[0m')

def is_ico_valid(ico1, ico2, ico3):
    print('>>> Start testing "The initial state after ICO should contain only 1 entry" <<<')

    if len(ico1) == 1 and len(ico1) == len(ico2) and len(ico2) == len(ico3) and ico1[0] == ico2[0] and ico2[0] == ico3[0]:
        print('\033[32m' + '[Test 1] - Initial ICO has one entry and they are all the same PASSED!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 1] - Initial ICO has one entry and they are all the same FAILED!' + '\033[0m')
        print(ico1)
        print(ico2)
        print(ico3)


def is_kth_block_state_same(state1, state2, state3, k):
    print(f'>>> Start testing "Whether the {k}th block state are the same" <<<')
    state1_ = sorted(state1)
    state2_ = sorted(state2)
    state3_ = sorted(state3)
    if (state1_ == state1):
        print('\033[32m' + '[Test 2] - the state sort for 0 PASSED!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 2] - the state sort for 0 FAILED!' + '\033[0m')

    if (state2_ == state2):
        print('\033[32m' + '[Test 2] - the state sort for 1 PASSED!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 2] - the state sort for 1 FAILED!' + '\033[0m')

    if (state3_ == state3):
        print('\033[32m' + '[Test 2] - the state sort for 2 PASSED!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 2] - the state sort for 2 FAILED!' + '\033[0m')

    if state1 == state2 and state2 == state3:
        print('\033[32m' + f'[Test 2] - the {k}th block state are the same PASSED!' + '\033[0m')
    else:
        print('\033[31m' + f'[Test 2] - the {k}th block state are the same FAILED!' + '\033[0m')
        print(state1)
        print(state2)
        print(state3)

def is_entry_number_great_equal_than_k(state, k):
    print(f'>>> Start testing "Whether the entry number is great equal than {k}" <<<')

    if len(state) >= k:
        print('\033[32m' + f'[Test 2] - the entry number is great equal than {k} PASSED!' + '\033[0m')
    else:
        print('\033[31m' + f'[Test 2] - the entry number is great equal than {k} FAILED!' + '\033[0m')
        print(state)


def is_state_evolve(state0, state10, state20):
    print(f'>>> Start testing "Whether the state evolves for 0th, 10th, and 20th states" <<<')
    state0_ = sorted(state0)
    state10_ = sorted(state10)
    state20_ = sorted(state20)

    if (state0_ == state0):
        print('\033[32m' + '[Test 3] - the state sort for 0th PASSED!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 3] - the state sort for 0th FAILED!' + '\033[0m')

    if (state10_ == state10):
        print('\033[32m' + '[Test 3] - the state sort for 10th PASSED!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 3] - the state sort for 10th FAILED!' + '\033[0m')

    if (state20_ == state20):
        print('\033[32m' + '[Test 3] - the state sort for 20th PASSED!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 3] - the state sort for 20th FAILED!' + '\033[0m')

    if state0 != state10 and state10 != state20 and state0 != state20:
        print('\033[32m' + '[Test 3] - the state evolves for 0th, 10th, and 20th states PASSED!' + '\033[0m')
    else:
        print('\033[31m' + '[Test 3] - the state evolves for 0th, 10th, and 20th states FAILED!' + '\033[0m')
        print(state0)
        print(state10)
        print(state20)


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

        longest_chain1 = json.loads(str(requests.get('http://127.0.0.1:7000/blockchain/longest-chain').content.decode('utf-8')))
        longest_chain2 = json.loads(str(requests.get('http://127.0.0.1:7001/blockchain/longest-chain').content.decode('utf-8')))
        longest_chain3 = json.loads(str(requests.get('http://127.0.0.1:7002/blockchain/longest-chain').content.decode('utf-8')))
        first_block_state1 = json.loads(str(requests.get('http://127.0.0.1:7000/blockchain/state?block=0').content.decode('utf-8')))
        first_block_state2 = json.loads(str(requests.get('http://127.0.0.1:7001/blockchain/state?block=0').content.decode('utf-8')))
        first_block_state3 = json.loads(str(requests.get('http://127.0.0.1:7002/blockchain/state?block=0').content.decode('utf-8')))
        tenth_block_state1 = json.loads(str(requests.get('http://127.0.0.1:7000/blockchain/state?block=10').content.decode('utf-8')))
        tenth_block_state2 = json.loads(str(requests.get('http://127.0.0.1:7001/blockchain/state?block=10').content.decode('utf-8')))
        tenth_block_state3 = json.loads(str(requests.get('http://127.0.0.1:7002/blockchain/state?block=10').content.decode('utf-8')))
        twentieth_block_state1 = json.loads(str(requests.get('http://127.0.0.1:7000/blockchain/state?block=20').content.decode('utf-8')))
        twentieth_block_state2 = json.loads(str(requests.get('http://127.0.0.1:7001/blockchain/state?block=20').content.decode('utf-8')))
        twentieth_block_state3 = json.loads(str(requests.get('http://127.0.0.1:7002/blockchain/state?block=20').content.decode('utf-8')))


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

        time.sleep(3)

        is_blockchain_valid(longest_chain1, longest_chain2, longest_chain3)
        is_ico_valid(first_block_state1, first_block_state2, first_block_state3)
        is_kth_block_state_same(first_block_state1, first_block_state2, first_block_state3, 1)
        is_kth_block_state_same(tenth_block_state1, tenth_block_state2, tenth_block_state3, 10)
        is_kth_block_state_same(twentieth_block_state1, twentieth_block_state2, twentieth_block_state3, 20)
        is_entry_number_great_equal_than_k(tenth_block_state1, 3)
        is_entry_number_great_equal_than_k(tenth_block_state2, 3)
        is_entry_number_great_equal_than_k(tenth_block_state3, 3)
        is_entry_number_great_equal_than_k(twentieth_block_state1, 3)
        is_entry_number_great_equal_than_k(twentieth_block_state2, 3)
        is_entry_number_great_equal_than_k(twentieth_block_state3, 3)
        is_state_evolve(first_block_state1, tenth_block_state1, twentieth_block_state1)
        is_state_evolve(first_block_state2, tenth_block_state2, twentieth_block_state2)
        is_state_evolve(first_block_state3, tenth_block_state3, twentieth_block_state3)

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

