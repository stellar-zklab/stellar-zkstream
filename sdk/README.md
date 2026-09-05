# @stellar-zklab/zkstream-sdk

TypeScript SDK for `stellar-zkstream`'s privacy-preserving payment streams on Stellar.

## Current Status — what's real

`StellarZkStreamClient` wraps `@stellar/stellar-sdk/contract`'s `Client` and makes real
simulate/sign/submit calls against a real Soroban RPC endpoint — the same integration
`../frontend/src/soroban.ts` uses, factored out into a reusable package. `createStream()`,
`withdraw()`, and `cancelStream()` build, sign, and submit real transactions gated by real
on-chain Groth16 proof verification; the other methods simulate against real on-chain
stream state.

Signing is dependency-injected rather than hard-wired to Freighter, so this SDK works with
any wallet adapter that can produce a signed transaction XDR string.

```ts
import { StellarZkStreamClient } from '@stellar-zklab/zkstream-sdk';
import freighter from '@stellar/freighter-api';

const zkstream = new StellarZkStreamClient({
  streamContractId: 'CDEJZ5GPOW5GMDTBMJ2WD7ENPKHBPETZFUGAZ76QMDGHT5LNAEG5TKH7',
  verifierId: 'CARWCSIHZ7HCXDCCLRN2JX7SYDAKMZXI53M6AGUUXPRLLT3UJ3WIDLIY',
  signTransaction: async (xdr, opts) => {
    const { signedTxXdr } = await freighter.signTransaction(xdr, opts);
    return signedTxXdr;
  },
});

const streamId = await zkstream.createStream({
  sender, recipient, token,
  totalAmount: 5_000_000n,
  startTime, cliffTime, endTime,
  cancelable: true,
  proof, publicInputs, // see "Proof generation" below
});
```

## Proof generation is not included

This SDK does not generate range or nullifier proofs — general in-browser proof
generation (bundling the circuit's wasm witness calculator + snarkjs prover) isn't built
anywhere in this repo yet. The frontend demo works around this by submitting one
precomputed proof for a fixed amount (see `../circuits/README.md` and
`../circuits/build/range_proof` for how it was generated via snarkjs against the real
`range_proof.circom` circuit). Callers of this SDK must supply their own real `proof` and
`publicInputs`, generated out-of-band the same way, until in-browser proving exists.

## Known design limitation

The range proof gates `createStream()` on `amount` being within `[min_amount,
max_amount]`, but `total_amount` is *also* passed as a plain, public argument to
`createStream()` (it has to be — the contract needs it to move real tokens). The contract
does not currently cross-check the proof's public inputs against that argument, so the
"privacy" the range proof provides doesn't actually hide the streamed amount today, since
it's already visible as a call argument. This is a real open design question, not
something this SDK works around silently.
