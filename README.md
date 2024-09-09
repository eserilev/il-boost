# Out-of-Protocol Inclusion Lists via Commit-Boost

*Many thanks to Eitan Seri-Levi, Drew van der Werff, Barnabé Monnot, and the entire Inclusion List Module via Commit-Boost team for feedback on this design document (feedback does not imply endorsement).*

Below, we discuss a [Commit-Boost](https://github.com/Commit-Boost/commit-boost-client) [module](https://commit-boost.github.io/commit-boost-client/category/developing) that allows a proposer of a slot to create a list of transactions that it wants to be included in the block produced for its slot while still outsourcing block construction via [MEV-Boost](https://boost.flashbots.net/). In this module, the proposer constructs an inclusion list and propagates it to one or more relays that enforce the inclusion list. The proposer expresses a certain value that it has for getting the transactions from the inclusion list included in the block. This design trades off the adoption of the out-of-protocol inclusion list with the effectiveness of the inclusion list. 

The inclusion list module uses [Commit-Boost](https://github.com/Commit-Boost/commit-boost-client) and the [Bolt builder constraints API](https://chainbound.github.io/bolt-docs/api/builder-api). A beacon node can treat this module as an MEV-Boost relay.

## Construction

The proposer of slot $N$ constructs a list of transactions that it wants to include in the block proposed during slot $N$. 

The list is constructed using the `get_filtered_transactions` function that takes as input a vector of `Transaction` objects; these are transactions that were pending in the mempool but not included in an earlier block and a `Block` that is the block proposed during slot $N-1$. For each of these transactions in `Transaction`, it checks whether it would have been possible to include the transaction based on two conditions: 1) does the transaction pay a strictly positive priority fee, and 2) is the gas limit of the transaction smaller or equal to the amount of gas leftover in the `Block`. The transaction is added to the inclusion list if these two conditions are satisfied.

The transactions included in the `Transaction` object are transactions that were seen by the proposer of slot $N$ at the beginning of the slot and were not included in any earlier block. Setting the freeze deadline of transactions to be included in the `Transaction` object earlier means the proposer only considers transactions that are more likely to be excluded. In contrast, it increases the average wait time for transactions to be included.

## Propagation

The proposer sends its inclusion list to the MEV-Boost relay(s) of its choosing as soon as it is constructed. Next to the inclusion list, the proposer forwards the value that it attaches to the inclusion list being satisfied.

## Auction impact

If the relay receives an inclusion list from a proposer, it first checks whether this proposer is indeed the proposer of the slot to which the inclusion list corresponds. Then, it broadcasts the inclusion list so that the builders who interact with the relay are aware of it and the value that the proposer attaches to it.

Each builder then builds blocks and submits bids to the relay in the auction. When the proposer calls `get_header`, the relay forwards the header corresponding to the highest adjusted bid, where the highest adjusted bid is computed as follows. Note that the highest adjusted bid is purely to incorporate the value that the proposer assigns to the inclusion list, and it is not the value that the proposer receives. The relay should also forward the bid to the proposer.

$$
\text{Highest Adjusted Bid} = \max\{\text{IL-Satisfying Bid} + \text{Proposer IL Value}, \enskip  \text{Non-IL-Satisfying Bid} \}
$$

Along with the header, the relay also forwards whether the inclusion list has been satisfied or not. This is useful if the proposer wants to compare bids across relays.

The relay could either compute the highest adjusted bid [optimistically](https://github.com/michaelneuder/optimistic-relay-documentation/blob/main/proposal.md) by letting builders state whether they satisfy the IL or not or non-optimistically by 1) ensuring that the block is valid and 2) ensuring that all transactions from the inclusion list are included in the block.

## Properties

The inclusion list can have various properties, as described in the research on in-protocol inclusion lists. This section details the design trade-offs for this inclusion list module.

### Optimistic vs. Non-Optimistic

The proposer could either enforce the inclusion list optimistically or non-optimistically. In this design, we have chosen optimistic enforcement. This means that the proposer does not require proof from the builder or relay that the block that corresponds to the header that the proposer commits to includes the transactions from the inclusion list. Instead, the proposer trusts that the relay correctly reports whether or not the header that the proposer receives corresponds to a payload that satisfies the inclusion list.

If the relay makes an error and says a payload satisfies the inclusion list, while this is not the case, the relay must compensate the proposer for the value that the proposer had attached to its inclusion list being validated.

We chose optimistic enforcement because the proposer must trust the relay, regardless of whether it uses the inclusion list module, to ensure that the proposer receives the value of the bid it commits to. This means that either the relay must check that the block is valid and pay the bid, or it must guarantee payment.

The inclusion list module does not introduce qualitatively new trust assumptions. The proposer must now trust the relay that the block is valid and satisfies the inclusion. If one of these two conditions is not met, the relay must guarantee payment to the proposer.

Using a proof-based system in which the proposer only signs the header if it knows that the inclusion list has been satisfied would ensure that the proposer knows whether the inclusion list is satisfied if the block is valid. However, the block could still be invalid. Moreover, the proof increases latency, which would result in a reduction of expected profit and would, therefore, likely lead to reduced adoption.

### [Forward](https://notes.ethereum.org/@fradamt/forward-inclusion-lists) vs. Spot

The inclusion list applies to the slot of the proposer who constructs it and could thus be deemed a spot inclusion list. Spot inclusion lists are known to be not incentive compatible as the proposer restricts the possible blocks that can be built for its slot and, therefore, also restricts the maximum value it can extract. 

This design, however, assumes that a proposer has some private value for a block to satisfy its inclusion list. The proposer communicates this private value to the relay alongside its inclusion list, and this value is then used to compute the highest adjusted bid, as detailed earlier. Given this private valuation, the inclusion list is incentive-compatible.

The private valuation could originate from various sources; for example, the validator may value contributing to the network's credible neutrality by including all transactions it sees.

### Conditional vs. [Unconditional](https://ethresear.ch/t/unconditional-inclusion-lists/18500)

The inclusion list is applied unconditionally, regardless of congestion. Hence, if the block is full, the inclusion list is still only satisfied if all transactions from the inclusion list are included in the block. This is different from the conditional inclusion lists that are often discussed for in-protocol inclusion lists.

This design chooses unconditional inclusion lists because it wants to convey the proposer's preference for certain transactions as clearly as possible. A proposer may want certain transactions included regardless of whether other transactions can be included.

Finally, the inclusion list is conditional on the relative value between a payload that satisfies it and one that does not. Times of congestion may also be accompanied by higher priority fees; hence, in practice, during congestion, the bid of a payload that does not satisfy the inclusion list may be higher than the sum of the bid of a payload that satisfies the inclusion list and the private value.

### [Uncrowdability](https://ethresear.ch/t/uncrowdable-inclusion-lists-the-tension-between-chain-neutrality-preconfirmations-and-proposer-commitments/19372)

Uncrowdability means that an inclusion list will be used for credible neutrality and not for other profit-driven motives. This is not a concern with out-of-protocol inclusion lists via Commit-Boost because these inclusion lists do not have any special properties that make it more attractive to use for profit-driven motives as there are other modules built on Commit-Boost that are specifically designed for things like preconfirmations. Unlike in-protocol inclusion lists, the block validity is not tied to whether the inclusion list is satisfied. If a proposer wants to pursue profits, they could use another module on Commit-Boost rather than the inclusion list module. 

## Evaluation

Since the inclusion list is validated optimistically, there must be a check to determine whether it was indeed satisfied. This can be the same check as determining whether the block indeed paid the validator the bid. 

## Timing and Double Submitting

Unlike in-protocol inclusion lists, there is no consensus on the inclusion list, so there are no strict timing deadlines. When the proposer calls `get_header`, the relay communicates whether the payload satisfies the inclusion list or not. If the inclusion list was not submitted on time, the relay communicates that the payload does not satisfy the inclusion list.

If the are two or more inclusion lists that the proposer has specified, the relay must not be able to be grieved and (socially) forced to pay out to a proposer. Therefore, in this design, the relay will return that the payload satisfies the inclusion list as long as the payload satisfies an inclusion list that the proposer has broadcasted.

## Default Preference Value

The proposer specifies a value that it attaches to the inclusion list being satisfied by a payload. This value functions as the maximum value a proposer may lose if it made the inclusion list for purely altruistic purposes. So, this value is similar to the [min-bid](https://writings.flashbots.net/the-cost-of-resilience) parameter in MEV-Boost. As Data Always has shown, it is very important to set this value properly since it is a careful balance between the cost of altruism and the effectiveness of the policy.

### [Encrypted Mempool](https://joncharbonneau.substack.com/p/encrypted-mempools) / min-bid Model

Encrypted mempools increase the [cost of censorship](https://cdn.prod.website-files.com/642f3d0236c604d1022330f2/6499f35e0bd0f43471a95adc_MEV_Auctions_ArXiV_6.pdf) from the priority fee a transaction pays to the priority fees that all transactions pay since an adversary cannot selectively exclude one specific transaction but must exclude all transactions. This can be seen as a large constant addition to the cost of censorship. The out-of-protocol inclusion lists could emulate this model by setting the default preference value to a fixed constant, much like the min-bid parameter. This would also form a clear maximum loss that an altruistic proposer may face. 

### [Multiple Proposers](https://cdn.prod.website-files.com/642f3d0236c604d1022330f2/6499f35e0bd0f43471a95adc_MEV_Auctions_ArXiV_6.pdf) / local block value boost Model

Multiple proposers increase the cost of censorship from the priority fee to the priority fee multiplied by the number of proposers since an adversary must bribe all proposers to exclude the transaction. The out-of-protocol inclusion list could achieve a similar cost of censorship by setting the default preference value to be a multiple of the priority fees of the transactions included in the inclusion list. This is more similar to the [`local-block-value-boost` parameter](https://docs.prylabs.network/docs/advanced/builder#prioritizing-local-blocks) implemented by clients.

I would argue that this design should use the encrypted mempool / min-bid model since it is easier for the proposer to reason about a correct value and because clients are implementing a maximum value for the local block value boost based on [Data Always’ recent post](https://hackmd.io/@dataalways/censorship-resistance-today).


# How to run 

The following commands will run the IL-Boost module.

`cargo build --release`

`sudo docker compose -f cb.docker-compose.yml up -d`

`sudo docker build -t il-boost . -f ./Dockerfile`

But first you will need to update the `cb.docker-compose.yml` and `cb-config.toml` files

For `cb.docker-compose.yml` you'll need to change the following values

- `${YOUR_PATH_TO_KEYS_DIR}` should be replaced with the relative/absolute path to your keys directory
- `${YOUR_PATH_TO_SERETS_DIR}` should be replaced with the relative/absolute path to your secrets directory
- `${JWT}` should be replaced with your nodes JWT key. Note that this value needs to be replaced in two places

For `cb-config.toml` please see the list of example configurations and update them accordingly

## EL configs

Make sure to enable the following web api features

`admin,engine,net,eth,web3,debug,txpool`

i.e `--http.api=admin,engine,net,eth,web3,debug,txpool` in Geth

## CL/VC configs

Make sure to enable external block building capabilities and point the block builder URL to your local Commit-Boost PBS module