import React, { useState } from 'react';
import './index.css';
import {
  connectWallet,
  verifyRealProofOnChain,
  createRealDemoStream,
  getRealStream,
  DEMO_STREAM_AMOUNT_STROOPS,
  STREAM_CONTRACT_ID,
  RANGE_PROOF_VERIFIER_ID,
} from './soroban';

// This UI is wired to REAL, deployed Stellar testnet contracts (see soroban.ts and
// deployments/testnet.json at the repo root) — not a mock. Two things remain honestly
// simulated rather than real, and are labeled as such below: (1) creating a stream only
// works for one fixed amount (0.5 XLM), because it submits the one precomputed real ZK
// proof this repo has generated so far — general in-browser proof generation for an
// arbitrary amount is real future work, not built yet; (2) recipient addresses are not
// validated against a real Stellar keypair before submission.

interface StreamItem {
  id: number;
  type: 'outflow' | 'inflow';
  counterparty: string;
  totalAmount: number;
  vestedAmount: number;
  status: string;
}

export const App: React.FC = () => {
  const [walletAddress, setWalletAddress] = useState<string | null>(null);
  const [walletError, setWalletError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'create' | 'outflow'>('create');
  const [recipient, setRecipient] = useState('');
  const [loading, setLoading] = useState(false);
  const [verifying, setVerifying] = useState(false);

  const [logs, setLogs] = useState<string[]>([
    `[REAL] This app talks to real deployed contracts on Stellar testnet — stream: ${STREAM_CONTRACT_ID}`,
  ]);

  const [streams, setStreams] = useState<StreamItem[]>([]);

  const appendLog = (line: string) => setLogs((prev) => [...prev, line]);

  const handleConnect = async () => {
    setWalletError(null);
    try {
      const address = await connectWallet();
      setWalletAddress(address);
      appendLog(`[REAL] Connected real Freighter wallet: ${address.substring(0, 8)}...`);
    } catch (err: any) {
      setWalletError(err.message ?? String(err));
      appendLog(`[REAL] Wallet connection failed: ${err.message ?? err}`);
    }
  };

  const handleVerifyOnChain = async () => {
    setVerifying(true);
    appendLog(`[REAL] Calling vrfy_prf on the real deployed verifier (${RANGE_PROOF_VERIFIER_ID.substring(0, 8)}...) with a real Groth16 proof...`);
    try {
      const result = await verifyRealProofOnChain();
      appendLog(`[REAL] Testnet responded: vrfy_prf() = ${result}. This is a live simulateTransaction call, not a mock.`);
    } catch (err: any) {
      appendLog(`[REAL] On-chain verification call failed: ${err.message ?? err}`);
    } finally {
      setVerifying(false);
    }
  };

  const handleCreateStream = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!recipient) return;

    if (!walletAddress) {
      appendLog('[DEMO] No wallet connected — showing a local-only illustrative row. Connect a real wallet above to submit a real transaction.');
      setStreams((prev) => [
        { id: prev.length, type: 'outflow', counterparty: recipient.trim(), totalAmount: 0.5, vestedAmount: 0, status: 'Demo — not a real stream' },
        ...prev,
      ]);
      setRecipient('');
      setActiveTab('outflow');
      return;
    }

    setLoading(true);
    appendLog(`[REAL] Submitting a real create_stream transaction for ${Number(DEMO_STREAM_AMOUNT_STROOPS) / 1e7} XLM (fixed demo amount — see soroban.ts) — this needs your wallet signature.`);
    try {
      const streamId = await createRealDemoStream(walletAddress, recipient.trim());
      appendLog(`[REAL] Transaction confirmed. Real stream #${streamId} created on testnet.`);
      const stream = await getRealStream(streamId);
      setStreams((prev) => [
        {
          id: streamId,
          type: 'outflow',
          counterparty: stream.recipient,
          totalAmount: Number(stream.total_amount) / 1e7,
          vestedAmount: 0,
          status: 'Real — confirmed on testnet',
        },
        ...prev,
      ]);
      setRecipient('');
      setActiveTab('outflow');
    } catch (err: any) {
      appendLog(`[REAL] create_stream failed: ${err.message ?? err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ minHeight: '100vh', backgroundColor: '#090d16', color: '#e2e8f0' }}>
      <div style={{ background: 'linear-gradient(135deg, #0f766e, #065f46)', color: '#fff', padding: '0.65rem 1.5rem', fontSize: '0.85rem', fontWeight: 600, textAlign: 'center' }}>
        ✓ Wired to real deployed testnet contracts. Creating a stream is limited to one fixed, pre-proven amount (0.5 XLM) — see banner in soroban.ts for why.
      </div>
      <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '2rem 1.5rem', display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>

        <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#111827', padding: '1rem 1.5rem', borderRadius: '10px', border: '1px solid #1f2937' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
            <h1 style={{ fontSize: '1.25rem', fontWeight: 700, margin: 0, color: '#06b6d4' }}>stellar-zkstream</h1>
            <span style={{ fontSize: '0.75rem', background: 'rgba(15, 118, 110, 0.2)', color: '#5eead4', padding: '0.2rem 0.5rem', borderRadius: '4px', border: '1px solid rgba(15, 118, 110, 0.4)', fontWeight: 600 }}>
              Testnet — Real Contracts
            </span>
          </div>

          <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
            <button
              onClick={handleVerifyOnChain}
              disabled={verifying}
              style={{ padding: '0.5rem 1rem', background: '#1e293b', color: '#5eead4', border: '1px solid #334155', borderRadius: '6px', cursor: verifying ? 'wait' : 'pointer', fontWeight: 600, fontSize: '0.85rem' }}
            >
              {verifying ? 'Verifying on-chain...' : 'Verify Real Proof On-Chain'}
            </button>
            <button
              onClick={handleConnect}
              style={{ padding: '0.5rem 1rem', background: '#1e293b', color: '#38bdf8', border: '1px solid #334155', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '0.85rem' }}
            >
              {walletAddress ? `${walletAddress.substring(0, 6)}...${walletAddress.substring(walletAddress.length - 4)}` : 'Connect Freighter Wallet'}
            </button>
          </div>
        </header>

        {walletError && (
          <div style={{ background: 'rgba(190, 18, 60, 0.15)', border: '1px solid rgba(190, 18, 60, 0.4)', color: '#fda4af', padding: '0.75rem 1rem', borderRadius: '8px', fontSize: '0.85rem' }}>
            {walletError}
          </div>
        )}

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.5rem' }}>

          <section style={{ background: '#111827', padding: '1.75rem', borderRadius: '10px', border: '1px solid #1f2937', display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>
            <div style={{ display: 'flex', gap: '1rem', borderBottom: '1px solid #1f2937', paddingBottom: '0.75rem' }}>
              <button
                onClick={() => setActiveTab('create')}
                style={{ background: 'none', border: 'none', color: activeTab === 'create' ? '#06b6d4' : '#94a3b8', borderBottom: activeTab === 'create' ? '2px solid #06b6d4' : 'none', paddingBottom: '0.5rem', cursor: 'pointer', fontWeight: 600, fontSize: '0.9rem' }}
              >
                Create Stream
              </button>
              <button
                onClick={() => setActiveTab('outflow')}
                style={{ background: 'none', border: 'none', color: activeTab === 'outflow' ? '#06b6d4' : '#94a3b8', borderBottom: activeTab === 'outflow' ? '2px solid #06b6d4' : 'none', paddingBottom: '0.5rem', cursor: 'pointer', fontWeight: 600, fontSize: '0.9rem' }}
              >
                Streams ({streams.length})
              </button>
            </div>

            {activeTab === 'create' ? (
              <form onSubmit={handleCreateStream} style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                <div>
                  <label style={{ display: 'block', fontSize: '0.8rem', fontWeight: 600, color: '#94a3b8', marginBottom: '0.4rem' }}>Recipient Address</label>
                  <input
                    type="text"
                    placeholder="G... (a real testnet account, if using a real wallet)"
                    value={recipient}
                    onChange={(e) => setRecipient(e.target.value)}
                    style={{ width: '100%', padding: '0.75rem 1rem', background: '#090d16', border: '1px solid #374151', color: '#f8fafc', borderRadius: '6px', fontSize: '0.9rem', outline: 'none', boxSizing: 'border-box' }}
                  />
                </div>

                <div style={{ fontSize: '0.8rem', color: '#64748b' }}>
                  Stream amount is fixed at {Number(DEMO_STREAM_AMOUNT_STROOPS) / 1e7} XLM for this demo — see the banner at the top of soroban.ts for why.
                </div>

                <button
                  type="submit"
                  disabled={loading || !recipient}
                  style={{ padding: '0.85rem', background: loading ? '#374151' : '#0891b2', color: '#ffffff', border: 'none', borderRadius: '6px', cursor: loading ? 'wait' : 'pointer', fontWeight: 600, fontSize: '0.9rem', marginTop: '0.5rem' }}
                >
                  {loading ? 'Submitting real transaction...' : walletAddress ? 'Create Real Stream (Signs & Submits)' : 'Add Demo Row (Connect Wallet for Real Tx)'}
                </button>
              </form>
            ) : (
              <div style={{ overflowX: 'auto' }}>
                {streams.length === 0 ? (
                  <p style={{ color: '#64748b', textAlign: 'center', padding: '1.5rem' }}>No streams yet — use "Create Stream" above.</p>
                ) : (
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.85rem' }}>
                  <thead>
                    <tr style={{ borderBottom: '1px solid #1f2937', textAlign: 'left', color: '#64748b' }}>
                      <th style={{ padding: '0.5rem' }}>ID</th>
                      <th style={{ padding: '0.5rem' }}>Recipient</th>
                      <th style={{ padding: '0.5rem' }}>Total</th>
                      <th style={{ padding: '0.5rem' }}>Status</th>
                    </tr>
                  </thead>
                  <tbody>
                    {streams.map(s => (
                      <tr key={s.id} style={{ borderBottom: '1px solid #111827' }}>
                        <td style={{ padding: '0.75rem 0.5rem', fontWeight: 600, color: '#38bdf8' }}>#{s.id}</td>
                        <td style={{ padding: '0.75rem 0.5rem', fontFamily: 'monospace' }}>{s.counterparty.substring(0, 8) || 'n/a'}...</td>
                        <td style={{ padding: '0.75rem 0.5rem' }}>{s.totalAmount} XLM</td>
                        <td style={{ padding: '0.75rem 0.5rem', color: s.status.startsWith('Real') ? '#5eead4' : '#fbbf24' }}>{s.status}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                )}
              </div>
            )}
          </section>

          <section style={{ background: '#090d16', padding: '1.5rem', borderRadius: '10px', border: '1px solid #1f2937', display: 'flex', flexDirection: 'column' }}>
            <h2 style={{ fontSize: '0.95rem', fontWeight: 600, color: '#94a3b8', margin: '0 0 1rem 0' }}>
              Activity Log
            </h2>

            <div style={{ background: '#030712', padding: '1.25rem', borderRadius: '8px', border: '1px solid #111827', fontFamily: 'Fira Code, monospace', fontSize: '0.8rem', color: '#10b981', flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '0.6rem' }}>
              {logs.map((log, idx) => (
                <div key={idx} style={{ color: log.startsWith('[REAL]') ? '#5eead4' : '#fbbf24' }}>
                  {log}
                </div>
              ))}
            </div>
          </section>

        </div>

      </div>
    </div>
  );
};
export default App;
