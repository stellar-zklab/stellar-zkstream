pragma circom 2.1.6;

include "node_modules/circomlib/circuits/poseidon.circom";

/*
 * StreamNullifier
 *
 * Proves knowledge of `secret` that produced `nullifier_hash`
 * without revealing the secret. Prevents double-withdrawal.
 *
 * Private: secret
 * Public:  stream_id, nullifier_hash
 *
 * Constraint: nullifier_hash == Poseidon(secret, stream_id)
 */
template StreamNullifier() {
    signal input secret;
    signal input stream_id;
    signal input nullifier_hash;

    component h = Poseidon(2);
    h.inputs[0] <== secret;
    h.inputs[1] <== stream_id;
    nullifier_hash === h.out;
}

component main {public [stream_id, nullifier_hash]} = StreamNullifier();
