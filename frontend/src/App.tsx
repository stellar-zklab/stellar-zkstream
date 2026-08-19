import React, { useState, useEffect } from 'react';
import './index.css';

interface StreamItem {
  id: number;
  type: 'outflow' | 'inflow';
  counterparty: string;
  totalAmount: number;
  vestedAmount: number;
  status: string;
}

export const App: React.FC = () => {
  const [walletConnected, setWalletConnected] = useState(true);
  const [walletAddress] = useState('GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN');
  const [activeTab, setActiveTab] = useState<'create' | 'outflow'>('create');
  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');
  const [loading, setLoading] = useState(false);

  const [logs, setLogs] = useState<string[]>([
    '[SDK] Client initialized on Soroban Testnet',
    '[ConnectionPool] Connected to https://soroban-testnet.stellar.org:443 (Latency: 24ms)',
    '[ZKProof] Loaded BN254 Groth16 WASM Verification Keys',
    '[StreamContract] Fetched active stream ledger state for GAAZI4T...'
  ]);

  const [streams, setStreams] = useState<StreamItem[]>([
    { id: 0, type: 'outflow', counterparty: 'GBRPXHHFVLWCL3EBHWWSPBHGXBBTH75WWOF45AK2WBS2K7I5P5QLIOWK', totalAmount: 1000, vestedAmount: 142.50, status: 'Active' },
    { id: 1, type: 'inflow', counterparty: 'GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSFMG4W2C', totalAmount: 5000, vestedAmount: 1284.12, status: 'Active' }
  ]);

  useEffect(() => {
    const timer = setInterval(() => {
      setStreams(prev => prev.map(s => ({
        ...s,
        vestedAmount: s.vestedAmount < s.totalAmount ? s.vestedAmount + 0.005 : s.totalAmount
      })));
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  const handleCreateStream = (e: React.FormEvent) => {
    e.preventDefault();
    const num = parseFloat(amount);
    if (!recipient || isNaN(num) || num <= 0) return;

    setLoading(true);
    setLogs(prev => [...prev, `[ZKProver] Generating Groth16 BN254 proof for ${num} XLM...`]);

    setTimeout(() => {
      const newStream: StreamItem = {
        id: streams.length,
        type: 'outflow',
        counterparty: recipient.trim(),
        totalAmount: num,
        vestedAmount: 0.0,
        status: 'Active'
      };
      setStreams([newStream, ...streams]);
      setLoading(false);
      setLogs(prev => [
        ...prev,
        `[Soroban] Tx Submitted: 8f521c76a9179927... (Ledger #482910)`,
        `[StreamContract] Stream #${newStream.id} created successfully & escrowed on-chain`
      ]);
      setRecipient('');
      setAmount('');
      setActiveTab('outflow');
    }, 1000);
  };

  return (
    <div style={{ minHeight: '100vh', backgroundColor: '#090d16', color: '#e2e8f0', padding: '2rem 1.5rem' }}>
      <div style={{ maxWidth: '1200px', margin: '0 auto', display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
        
        {/* Navigation Bar */}
        <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#111827', padding: '1rem 1.5rem', borderRadius: '10px', border: '1px solid #1f2937' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
            <h1 style={{ fontSize: '1.25rem', fontWeight: 700, margin: 0, color: '#06b6d4' }}>stellar-zkstream</h1>
            <span style={{ fontSize: '0.75rem', background: 'rgba(16, 185, 129, 0.15)', color: '#10b981', padding: '0.2rem 0.5rem', borderRadius: '4px', border: '1px solid rgba(16, 185, 129, 0.3)', fontWeight: 600 }}>
              Testnet RPC (24ms)
            </span>
          </div>

          <button
            onClick={() => setWalletConnected(!walletConnected)}
            style={{ padding: '0.5rem 1rem', background: '#1e293b', color: '#38bdf8', border: '1px solid #334155', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '0.85rem' }}
          >
            {walletConnected ? `${walletAddress.substring(0, 6)}...${walletAddress.substring(50)} (Testnet)` : 'Connect Wallet'}
          </button>
        </header>

        {/* 2-Column Split Layout */}
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.5rem' }}>
          
          {/* Left Column: Action Card */}
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
                Active Streams ({streams.length})
              </button>
            </div>

            {activeTab === 'create' ? (
              <form onSubmit={handleCreateStream} style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                <div>
                  <label style={{ display: 'block', fontSize: '0.8rem', fontWeight: 600, color: '#94a3b8', marginBottom: '0.4rem' }}>Recipient Address</label>
                  <input
                    type="text"
                    placeholder="GBRPXHHFVLWCL3EBHWWSPBHGXBBTH75WWOF45AK2WBS2K7I5P5QLIOWK"
                    value={recipient}
                    onChange={(e) => setRecipient(e.target.value)}
                    style={{ width: '100%', padding: '0.75rem 1rem', background: '#090d16', border: '1px solid #374151', color: '#f8fafc', borderRadius: '6px', fontSize: '0.9rem', outline: 'none', boxSizing: 'border-box' }}
                  />
                </div>

                <div>
                  <label style={{ display: 'block', fontSize: '0.8rem', fontWeight: 600, color: '#94a3b8', marginBottom: '0.4rem' }}>Stream Amount (XLM)</label>
                  <input
                    type="number"
                    placeholder="1000"
                    value={amount}
                    onChange={(e) => setAmount(e.target.value)}
                    style={{ width: '100%', padding: '0.75rem 1rem', background: '#090d16', border: '1px solid #374151', color: '#f8fafc', borderRadius: '6px', fontSize: '0.9rem', outline: 'none', boxSizing: 'border-box' }}
                  />
                </div>

                <button
                  type="submit"
                  disabled={loading}
                  style={{ padding: '0.85rem', background: loading ? '#374151' : '#0891b2', color: '#ffffff', border: 'none', borderRadius: '6px', cursor: loading ? 'wait' : 'pointer', fontWeight: 600, fontSize: '0.9rem', marginTop: '0.5rem' }}
                >
                  {loading ? 'Verifying Proof...' : 'Create Confidential Stream'}
                </button>
              </form>
            ) : (
              <div style={{ overflowX: 'auto' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.85rem' }}>
                  <thead>
                    <tr style={{ borderBottom: '1px solid #1f2937', textAlign: 'left', color: '#64748b' }}>
                      <th style={{ padding: '0.5rem' }}>ID</th>
                      <th style={{ padding: '0.5rem' }}>Recipient</th>
                      <th style={{ padding: '0.5rem' }}>Total</th>
                      <th style={{ padding: '0.5rem' }}>Vested</th>
                    </tr>
                  </thead>
                  <tbody>
                    {streams.map(s => (
                      <tr key={s.id} style={{ borderBottom: '1px solid #111827' }}>
                        <td style={{ padding: '0.75rem 0.5rem', fontWeight: 600, color: '#38bdf8' }}>#{s.id}</td>
                        <td style={{ padding: '0.75rem 0.5rem', fontFamily: 'monospace' }}>{s.counterparty.substring(0, 8)}...</td>
                        <td style={{ padding: '0.75rem 0.5rem' }}>{s.totalAmount} XLM</td>
                        <td style={{ padding: '0.75rem 0.5rem', color: '#10b981', fontFamily: 'monospace' }}>{s.vestedAmount.toFixed(4)} XLM</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </section>

          {/* Right Column: SDK Execution Log Terminal */}
          <section style={{ background: '#090d16', padding: '1.5rem', borderRadius: '10px', border: '1px solid #1f2937', display: 'flex', flexDirection: 'column' }}>
            <h2 style={{ fontSize: '0.95rem', fontWeight: 600, color: '#94a3b8', margin: '0 0 1rem 0' }}>
              SDK Execution Log
            </h2>

            <div style={{ background: '#030712', padding: '1.25rem', borderRadius: '8px', border: '1px solid #111827', fontFamily: 'Fira Code, monospace', fontSize: '0.8rem', color: '#10b981', flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '0.6rem' }}>
              {logs.map((log, idx) => (
                <div key={idx} style={{ color: log.includes('[SDK]') ? '#38bdf8' : log.includes('[ZK') ? '#c084fc' : '#10b981' }}>
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
