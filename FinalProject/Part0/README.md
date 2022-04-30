# Part 0: Overview
In this project, you will use Solidity to create synthetic assets and develop a decentralized trading platform to enable exchanges among these assets on the Ethereum testnet. 

**Synthetic assets** are tokenized derivatives that produce the same value as another asset. It enables you to trade some assets without holding the asset itself. For instance, on Ethereum, you can trade synthetic assets representing fiat currencies (e.g. synthetic USD), other cryptocurrencies like BTC, even stocks (e.g. synthetic TSLA), which behave like the underlying asset by tracking its price using **data oracles** (will be explained in Part 1). 

We want to create a decentralized system to mint, manage, exchange synthetic assets, here are several example use cases to illustrate the application:

1. Marry wants to invest in Tesla stock with Ethereum, she mints synthetic Tesla (sTSLA) tokens by sending EUSD (ethereum stable coin) as collateral.
2. Linda owns lots of synthetic Binance Coins (sBNB) and sTSLA tokens and puts these tokens in a liquidity pool for rewards.
3. Tom owns lots of sBNB and wants to exchange them for sTSLA.


## Participants

In the above examples, three people involved represent three participants in our system, they are

* **Minter**: create Collateralized debt positions (CDP) in order to obtain newly minted tokens of a synthetic asset. CDPs can accept collateral in the form of EUSD and must maintain a collateral ratio above the minimum rate.
* **Liquidity provider**: add tokens to the corresponding pool, which increases liquidity for that market. 
* **Trader**: buy and sell synthetic tokens through a Uniswap-like protocol

Do not worry if you can not fully understand the role of each participant and some terminologies now, they will be introduced in more detail in subsequent parts. 

## Tokens

As suggested in the examples, three tokens will be used in our system. (ETH is not listed, but will also be used to pay gas fees.)

| Token Name | Token Symbol | Function | Type |
| -------- | -------- | -------- | -------- |
|Ethereum USD| EUSD | stable coin | ERC20 token|
|Synthetic Binance| sBNB | synthetic asset | ERC20 token|
|Synthetic TSLA| sTSLA| synthetic asset | ERC20 token|

## Smart Contracts

The whole system consists of three smart contracts, each for one part.

| Contract | Function |  |
| -------- | -------- |-------- |
| PriceFeed     | An interface to get prices for synthetic assets from oracle     | Part 1|
|Mint | For CDP creation, management, and liquidation |Part 2|
|SynthSwap | A Uniswap-like automated market maker (AMM) protocol |Part 3





## Deploy Token Contracts
In this part, you need to create three tokens introduced before. These tokens all follow the ERC20 standards and use the interface provided by [OpenZeppelin](https://docs.openzeppelin.com/contracts/4.x/erc20). The smart contracts are provided in the following files:
```
contracts/EUSD.sol
contracts/sAsset.sol
```
1. Open [Remix](https://remix.ethereum.org/) in your web browser, this is an online IDE where you will write, test and deploy your smart contracts.
2. Add provided contracts in Remix and compile them in the *Solidity compiler* tab. 
1. Create EUSD by deploying `EUSD.sol`, this will create a contract deployment transaction. The information and status of the transaction will be displayed in the terminal.
2. Create sBNB and sTSLA by deploying `sAsset.sol` with corresponding parameters ``(name, symbol, initialSupply)``, `name` and `symbol` are provided in the token table, `initialSupply = 0`.
3. After a contract is successfully deployed, you can see the instance under *Deployed Contracts*, where you can get your contract address and interact with the contract manually (e.g. if you call the `balanceOf` function of EUSD and enter your account address, you will get the number of EUSD tokens as output). 

## Submission
You do not need to submit anything for this part, just make sure you are able to deploy and interact with your smart contracts. Test the main functionalities of the tokens like `transfer`, `transferFrom`, and `mint`, `burn` for sAsset. Note that we use [Access Control](https://docs.openzeppelin.com/contracts/4.x/access-control) in sAsset to govern who can mint and burn tokens. The contract creator can grant minter and burner roles to other accounts by calling the `grantRole` function. You can select different accounts under *Account* to test these functions.


 