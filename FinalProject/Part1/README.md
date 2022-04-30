# Part 1: Creating price feeds

To reflect the value of other assets, we need to first obtain price feeds before minting synthetic assets. While everyone can get the latest price of a stock on NASDAQ and put it on the chain, data consumers may not want to trust any single data provider.

**Data oracles** provide a decentralized and trustworthy way for blockchains to access external data sources. Resources external to the blockchain are considered "off-chain" while data stored on the blockchain is considered on-chain. Oracle is an additional piece of infrastructure to bridge the two environments.

In this part, we will use one of the most popular oracle solutions, [Chainlink](https://docs.chain.link/), to create price feeds for our synthetic tokens.

## Testnet and wallet
To access the service provided by chainlink in the easiest way, we will use a public blockchain to deploy our smart contracts. In part 0, remember that when deploying contracts, some accounts are automatically generated, each with 100 ETH. These accounts belong to a private testnet blockchain where we can test our applications locally but can not interact with other contracts online. In this part, we need to create some public accounts in **Kovan Testnet** and use **Metamask** to manage them.

1. Install [MetaMask](https://chrome.google.com/webstore/detail/metamask/nkbihfbeogaeaoehlefnkodbefgpgknn) on Chrome, follow the instructions on the app to create a new wallet. After entering the correct phrases, a new account will be created automatically. You can create any number of accounts by clicking the upper right icon and *Create Account*.
2. Switch to Kovan Testnet: click the *Ethereum Mainnet* at the top right corner of the wallet page and turn on the testnet list by setting *Show/hide test networks*. Switch the network to *Kovan Test Network*.
3. Get some free ETH: go to a [faucet](https://faucets.chain.link/) and enter your address, you will get 0.1 ETH for testing.
4. Open [Remix](https://remix.ethereum.org/) in your web browser, in the *Deploy & run transactions* tab, set the environment to *Injected Web3*. This will launch a popup page to connect with your wallet.

## Price feed interface
We have provided the interface of the price feed smart contract in `interfaces/IPriceFeed.sol`, you need to implement your `PriceFeed.sol` and deploy one instance for each synthetic asset to provide their prices with respect to USD. You can refer to [this tutorial](https://docs.chain.link/docs/get-the-latest-price/) for help. The proxy addresses of each asset in Kovan are provided below:

```
BNB  / USD: 0x8993ED705cdf5e84D0a3B754b5Ee0e1783fcdF16
TSLA / USD: 0xb31357d152638fd1ae0853d24b9Ea81dF29E3EF2
```
1. There is only one function defined in the interface, you are required to implement it to provide the requested information. You can design other parts of the contract as you like.
2. Deploy the price feed contract for each asset, test the interface and copy their addresses. Once the deployment transactions are confirmed, you are able to find the deployed contracts in [etherscan](https://kovan.etherscan.io/) with https://kovan.etherscan.io/address/{:your_contract_address}.

## Submission
Submit the addresses of two contracts (20 bytes value with 0x appended in front) in this form: [https://forms.gle/zxA9zrKZSybxPqzP8](https://forms.gle/zxA9zrKZSybxPqzP8). Once the contracts are deployed, you can copy the address from Remix - Deployed Contracts.