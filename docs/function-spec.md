# Description of Plats Network Runtime Pallets

Plats network runtime uses some custom logic for blockchain business

## [Task Pallet](../pallets/task/README.md)

The Task pallet handles logic for create campaign, reward campaign and claim campain

## [Governance Pallet]()

The Governance pallet: holder can use $PLAT to vote or propose something in network. DAO

## [Treasury Pallet]()

The Treasury pallet: make treasury balance for marketing, reward, ....

## [Multicurrency Pallet]()

The Multicurrency pallet handles logic for creating token if clients dont have own token , they want to create and release their token ( in this case, clients dont use $PLAT for creating task )

## Stablecoin Exchange and Multicurrency Exchange

When users are rewarded by clients, users maybe have a lot of tokens in our system. We have stablecoin exchange mechanism to convert tokens into USN (NEAR stablecoin)
