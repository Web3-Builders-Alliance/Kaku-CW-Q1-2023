# Yieldmos Compounding Outposts

The purpose of this repo is to develop and release a set of asset management/compounding contracts predicated upon the functionality enabled by the CSDK Authz module.

## Protocol Usage

The expected flow should go as such:

1. The user should generate their own set of [compounding preferences](./packages/utils/src/comp_prefs.rs) and have them stored wherever they expect to be broadcasting the compounding message (This could be with Yieldmos itself or in the dapp/browser or potentially on the user's own computer for use via the cli).
2. The comp prefs should be given to the outpost in the grants query with the outpost returning a list of the requisite grants that will be needed to fulfilled in order for the outpost be able to later compound for them according to their comp prefs.
3. The user should grant the previously noted Authz grants to the outpost contract's adress.
4. The outpost's compound message can now be called whenever the compounding of rewards should occur.

## Outposts Progress

| Chain ID    | Rewards        | Status                                        | Address                                                                                                                                                                       |
| ----------- | -------------- | --------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `juno-1`    | `staking`      | [`working`](./contracts/junostake/README.md)  | [juno1qq0ra05mwvq7tkwndr8jxzdx5ragmxkp3ezqy8n49s332ap3ts9s9yvtyq](https://www.mintscan.io/juno/wasm/contract/juno1qq0ra05mwvq7tkwndr8jxzdx5ragmxkp3ezqy8n49s332ap3ts9s9yvtyq) |
| `juno-1`    | `wynd staking` | [`working`](./contracts/wyndstake/README.md)  | [juno15uaxh42m20f8d8072k2gghk08hjrr6jrccm0vdvkah0u0ksq997q8um4r5](https://www.mintscan.io/juno/wasm/contract/juno15uaxh42m20f8d8072k2gghk08hjrr6jrccm0vdvkah0u0ksq997q8um4r5) |
| `juno-1`    | `wynd lps`     | [`in progress`](./contracts/wyndlp/README.md) | [juno19wsa0zhjzzv5nz5ssmw5xl3sfl5sr3cye3rcv5g9ehs4lvx4acnsh4hjhn](https://www.mintscan.io/juno/wasm/contract/juno19wsa0zhjzzv5nz5ssmw5xl3sfl5sr3cye3rcv5g9ehs4lvx4acnsh4hjhn) |
| `osmosis-1` | `staking`      | `not started`                                 |                                                                                                                                                                               |
| `osmosis-1` | `lps`          | `not started`                                 |                                                                                                                                                                               |
| `osmosis-1` | `mars lending` | `not started`                                 |                                                                                                                                                                               |

Testing and the grants query are still in progress on all fronts. These must be codified before the v1 release.

## Outpost Contract Walkthrough

Walkthrough of the Outpost contract can be found [here](../capstone-walkthrough/capstone_presentation.mp4).
