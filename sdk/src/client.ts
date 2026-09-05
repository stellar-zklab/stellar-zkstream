/**
 * StellarZkStreamClient — TypeScript SDK for privacy-preserving payment streams.
 *
 * This wraps the same `@stellar/stellar-sdk/contract` Client the deployed frontend uses
 * (see ../../frontend/src/soroban.ts) — real simulate/sign/submit calls against a real
 * Soroban RPC endpoint. Signing is injected via `signTransaction` rather than hard-wired
 * to Freighter, so this SDK works with any wallet adapter that can produce a signed
 * transaction XDR string.
 *
 * This SDK does not generate range/nullifier proofs itself — general in-browser proof
 * generation (bundling the circuit's wasm witness calculator + snarkjs prover) isn't built
 * yet anywhere in this repo (see ../circuits/README.md and the frontend's own "one fixed
 * demo proof" limitation). Callers must supply a real proof and its public inputs, e.g.
 * generated out-of-band via snarkjs against ../circuits/build/range_proof.
 */
import { Client as ContractClient } from '@stellar/stellar-sdk/contract';

export type SignTransaction = (
  xdr: string,
  opts?: { network?: string; networkPassphrase?: string; accountToSign?: string }
) => Promise<string>;

export interface StellarZkStreamConfig {
  streamContractId: string;
  /** Only needed for verifyProof(); the address of a deployed zk_verifier instance
   * (range_verifier or nullifier_verifier — they're separate deployments with different
   * verification keys, see the contract's own initialize() doc comment). */
  verifierId?: string;
  rpcUrl?: string;
  networkPassphrase?: string;
  signTransaction: SignTransaction;
}

export interface OnChainStream {
  sender: string;
  recipient: string;
  token: string;
  total_amount: bigint;
  withdrawn_amount: bigint;
  start_time: bigint;
  cliff_time: bigint;
  end_time: bigint;
  active: boolean;
  cancelable: boolean;
}

export interface CreateStreamParams {
  sender: string;
  recipient: string;
  token: string;
  totalAmount: bigint;
  startTime: bigint;
  cliffTime: bigint;
  endTime: bigint;
  cancelable: boolean;
  proof: Uint8Array;
  publicInputs: Uint8Array[];
}

export interface WithdrawParams {
  streamId: bigint;
  caller: string;
  nullifierHash: Uint8Array;
  nullifierProof: Uint8Array;
  publicInputs: Uint8Array[];
}

export class StellarZkStreamClient {
  private streamContractId: string;
  private verifierId?: string;
  private rpcUrl: string;
  private networkPassphrase: string;
  private signTransaction: SignTransaction;

  constructor(config: StellarZkStreamConfig) {
    this.streamContractId = config.streamContractId;
    this.verifierId = config.verifierId;
    this.rpcUrl = config.rpcUrl ?? 'https://soroban-testnet.stellar.org';
    this.networkPassphrase = config.networkPassphrase ?? 'Test SDF Network ; September 2015';
    this.signTransaction = config.signTransaction;
  }

  private async getClient(contractId: string, publicKey?: string) {
    return ContractClient.from({
      contractId,
      networkPassphrase: this.networkPassphrase,
      rpcUrl: this.rpcUrl,
      publicKey,
      signTransaction: this.signTransaction,
    });
  }

  /** Real, live create_stream call — moves `params.sender`'s real tokens into the stream
   * contract and gates creation on a real range-proof verification on-chain. Requires
   * `params.sender` to sign. */
  async createStream(params: CreateStreamParams): Promise<bigint> {
    const client = await this.getClient(this.streamContractId, params.sender);
    const tx = await (client as any).create_stream(
      {
        sender: params.sender,
        recipient: params.recipient,
        token: params.token,
        total_amount: params.totalAmount,
        start_time: params.startTime,
        cliff_time: params.cliffTime,
        end_time: params.endTime,
        cancelable: params.cancelable,
        proof: Buffer.from(params.proof),
        public_inputs: params.publicInputs.map((b) => Buffer.from(b)),
      },
      { timeoutInSeconds: 1800 }
    );
    const sent = await tx.signAndSend();
    return sent.result as bigint;
  }

  /** Real, live withdraw call — gated on a real nullifier-proof verification on-chain, and
   * on that nullifier not having been used before. Requires `params.caller` to sign, and
   * must be the stream's registered recipient. */
  async withdraw(params: WithdrawParams): Promise<bigint> {
    const client = await this.getClient(this.streamContractId, params.caller);
    const tx = await (client as any).withdraw(
      {
        stream_id: params.streamId,
        caller: params.caller,
        nullifier_hash: Buffer.from(params.nullifierHash),
        nullifier_proof: Buffer.from(params.nullifierProof),
        public_inputs: params.publicInputs.map((b) => Buffer.from(b)),
      },
      { timeoutInSeconds: 1800 }
    );
    const sent = await tx.signAndSend();
    return sent.result as bigint;
  }

  /** Real, live cancel_stream call — splits vested/unvested funds between recipient and
   * sender per the contract's own on-chain math. Requires `caller` to sign and to be the
   * stream's sender. */
  async cancelStream(streamId: bigint, caller: string): Promise<void> {
    const client = await this.getClient(this.streamContractId, caller);
    const tx = await (client as any).cancel_stream({ stream_id: streamId, caller }, { timeoutInSeconds: 1800 });
    await tx.signAndSend();
  }

  /** Read-only: a real stream's on-chain state. */
  async getStream(streamId: bigint): Promise<OnChainStream> {
    const client = await this.getClient(this.streamContractId);
    const tx = await (client as any).get_stream({ stream_id: streamId });
    return tx.result as OnChainStream;
  }

  async getStreamsBySender(sender: string): Promise<bigint[]> {
    const client = await this.getClient(this.streamContractId);
    const tx = await (client as any).get_streams_by_sender({ sender });
    return tx.result as bigint[];
  }

  async getStreamsByRecipient(recipient: string): Promise<bigint[]> {
    const client = await this.getClient(this.streamContractId);
    const tx = await (client as any).get_streams_by_recipient({ recipient });
    return tx.result as bigint[];
  }

  /** Read-only: the real amount currently claimable on a stream, computed by the
   * contract's own on-chain vesting math. */
  async getClaimableAmount(streamId: bigint): Promise<bigint> {
    const client = await this.getClient(this.streamContractId);
    const tx = await (client as any).claimable_amount({ stream_id: streamId });
    return tx.result as bigint;
  }

  /** Read-only: verifies a real proof against the configured zk_verifier instance
   * directly, without going through the stream contract. Requires `verifierId` in the
   * client config. */
  async verifyProof(proof: Uint8Array, publicInputs: Uint8Array[]): Promise<boolean> {
    if (!this.verifierId) {
      throw new Error('verifyProof() requires verifierId in the client config');
    }
    const client = await this.getClient(this.verifierId);
    const tx = await (client as any).vrfy_prf({
      proof: Buffer.from(proof),
      public_inputs: publicInputs.map((b) => Buffer.from(b)),
    });
    return tx.result as boolean;
  }
}
