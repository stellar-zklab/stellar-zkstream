pragma circom 2.1.6;

include "node_modules/circomlib/circuits/poseidon.circom";
include "node_modules/circomlib/circuits/comparators.circom";

/*
 * RangeProof
 *
 * Proves: min_amount <= amount <= max_amount
 * without revealing the exact `amount` value.
 *
 * Used in stellar-zkstream create_stream() to prove amount > 0.
 *
 * Private: amount, salt
 * Public:  min_amount, max_amount, amount_commitment
 *
 * Constraints:
 *   1. amount_commitment == Poseidon(amount, salt)
 *   2. amount >= min_amount
 *   3. amount <= max_amount
 */
template RangeProof(n) {
    signal input amount;
    signal input salt;
    signal input min_amount;
    signal input max_amount;
    signal input amount_commitment;
    signal output valid;

    // 1. Commitment
    component h = Poseidon(2);
    h.inputs[0] <== amount;
    h.inputs[1] <== salt;
    amount_commitment === h.out;

    // 2. amount >= min
    component gte = GreaterEqThan(n);
    gte.in[0] <== amount;
    gte.in[1] <== min_amount;
    gte.out === 1;

    // 3. amount <= max
    component lte = LessEqThan(n);
    lte.in[0] <== amount;
    lte.in[1] <== max_amount;
    lte.out === 1;

    valid <== 1;
}

component main {public [min_amount, max_amount, amount_commitment]} = RangeProof(64);
