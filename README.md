# Inclusion List + Commit Boost

An implementation of an inclusion list sidecar using commit boost (https://github.com/Commit-Boost/commit-boost-client) and the Bolt builder constraints API (https://chainbound.github.io/bolt-docs/api/builder-api)

A beacon node client can treat this sidecar as a MEV-boost builder relay. This sidecar does the following

- Building + Forwarding inclusion list
- Verifying the inclusion list

## Building + forwarding the inclusion list

This sidecar is able to track upcoming proposer duties for any connected validators. If a proposer duty is found at slot N, it will preform the following actions at Slot N-1

- Identify a list of transactions that may be considered censored
- Using this list, generate an inclusion list that it will forward to the relay


## Verifying the inclusion list

At Slot N

- Call `get_payload_header_with_proof`
- Use the provided proof to verify that the transactions in the inclusion list are contained in the proof or in the previous slots execution payloaD
- If the above constraint is fulfilled, sign the execution payload and submit to the network 