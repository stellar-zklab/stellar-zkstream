import { describe, it, expect, vi } from 'vitest';
import { StellarZkStreamClient } from '../src/client';

const dummySign = vi.fn(async (xdr: string) => xdr);

describe('StellarZkStreamClient config', () => {
  it('constructs without a verifierId', () => {
    const client = new StellarZkStreamClient({
      streamContractId: 'CSTREAM00000000000000000000000000000000000000000000000000',
      signTransaction: dummySign,
    });
    expect(client).toBeInstanceOf(StellarZkStreamClient);
  });

  it('verifyProof() rejects clearly when verifierId was not configured', async () => {
    const client = new StellarZkStreamClient({
      streamContractId: 'CSTREAM00000000000000000000000000000000000000000000000000',
      signTransaction: dummySign,
    });
    await expect(client.verifyProof(new Uint8Array(), [])).rejects.toThrow('verifierId');
  });
});
