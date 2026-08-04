# Contributing to stellar-zkstream

## Stellar Drips Wave

This repo participates in the [Stellar Drips Wave](https://drips.network) program.

Browse [issues labelled `stellardrips`](https://github.com/stellar-zklab/stellar-zkstream/issues?q=label%3Astellardrips+is%3Aopen)
and apply via the Drips Wave dashboard.

| Label | Points |
|---|---|
| `trivial` | 100 pts |
| `medium-complexity` | 150 pts |
| `high-complexity` | 200 pts |

## Setup

```bash
git clone https://github.com/stellar-zklab/stellar-zkstream.git
cd stellar-zkstream
cargo test --all --features testutils
```

## PR Requirements

- Links the issue (`Closes #N`)
- Tests added and passing
- `cargo clippy` and `cargo fmt` clean
- CI passes
