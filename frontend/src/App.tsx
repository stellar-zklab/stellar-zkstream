import React, { useState } from 'react';
import './index.css';

// DEMO UI — there is no live backend behind this yet. No wallet is actually connected,
// no transaction is actually submitted, and the SDK log lines below are illustrative
// only, not real connection/proof output. See README.md for current project status.

interface StreamItem {
  id: number;
  type: 'outflow' | 'inflow';
  counterparty: string;
  totalAmount: number;
  vestedAmount: number;
  status: string;
}

export const App: React.FC = () => {
  const [walletConnected, setWalletConnected] = useState(false);
  const [activeTab, setActiveTab] = useState<'create' | 'outflow'>('create');
  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');
  const [loading, setLoading] = useState(false);

  const [logs, setLogs] = useState<string[]>([
    '[DEMO] No wallet connected, no backend deployed — everything below is a UI mockup.',
  ]);

  const [streams, setStreams] = useState<StreamItem[]>([]);

  const handleCreateStream = (e: React.FormEvent) => {
    e.preventDefault();
    const num = parseFloat(amount);
    if (!recipient || isNaN(num) || num <= 0) return;

    setLoading(true);
    setLogs(prev => [...prev, `[DEMO] Walking through the "create stream" UI for ${num} XLM — nothing is signed or submitted.`]);

    setTimeout(() => {
      const newStream: StreamItem = {
        id: streams.length,
        type: 'outflow',
        counterparty: recipient.trim(),
        totalAmount: num,
        vestedAmount: 0.0,
        status: 'Demo — not a real stream',
      };
      setStreams([newStream, ...streams]);
      setLoading(false);
      setLogs(prev => [
        ...prev,
        `[DEMO] Added a local, illustrative row to the table below. No Soroban transaction was created — there is no deployed stream contract this UI talks to yet.`,
      ]);
      setRecipient('');
      setAmount('');
      setActiveTab('outflow');
    }, 600);
  };

  return (
    <div style={{ minHeight: '100vh', backgroundColor: '#090d16', color: '#e2e8f0' }}>
      <div style={{ background: 'linear-gradient(135deg, #b45309, #92400e)', color: '#fff', padding: '0.65rem 1.5rem', fontSize: '0.85rem', fontWeight: 600, textAlign: 'center' }}>
        ⚠ DEMO MODE — no wallet is connected and no contract is deployed. Nothing below is real. See README for current status.
      </div>
      <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '2rem 1.5rem', display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>

        {/* Navigation Bar */}
        <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', background: '#111827', padding: '1rem 1.5rem', borderRadius: '10px', border: '1px solid #1f2937' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
            <h1 style={{ fontSize: '1.25rem', fontWeight: 700, margin: 0, color: '#06b6d4' }}>stellar-zkstream</h1>
            <span style={{ fontSize: '0.75rem', background: 'rgba(180, 83, 9, 0.2)', color: '#fbbf24', padding: '0.2rem 0.5rem', borderRadius: '4px', border: '1px solid rgba(180, 83, 9, 0.4)', fontWeight: 600 }}>
              No Backend Deployed
            </span>
          </div>

          <button
            onClick={() => setWalletConnected(!walletConnected)}
            style={{ padding: '0.5rem 1rem', background: '#1e293b', color: '#38bdf8', border: '1px solid #334155', borderRadius: '6px', cursor: 'pointer', fontWeight: 600, fontSize: '0.85rem' }}
          >
            {walletConnected ? 'Wallet (demo toggle only)' : 'Connect Wallet (not implemented)'}
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
                Create Stream (Demo)
              </button>
              <button
                onClick={() => setActiveTab('outflow')}
                style={{ background: 'none', border: 'none', color: activeTab === 'outflow' ? '#06b6d4' : '#94a3b8', borderBottom: activeTab === 'outflow' ? '2px solid #06b6d4' : 'none', paddingBottom: '0.5rem', cursor: 'pointer', fontWeight: 600, fontSize: '0.9rem' }}
              >
                Demo Streams ({streams.length})
              </button>
            </div>

            {activeTab === 'create' ? (
              <form onSubmit={handleCreateStream} style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                <div>
                  <label style={{ display: 'block', fontSize: '0.8rem', fontWeight: 600, color: '#94a3b8', marginBottom: '0.4rem' }}>Recipient Address</label>
                  <input
                    type="text"
                    placeholder="GABC...WXYZ (not used for anything real in this demo)"
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
                  {loading ? 'Running demo walkthrough...' : 'Run Demo Walkthrough (Not a Real Transaction)'}
                </button>
              </form>
            ) : (
              <div style={{ overflowX: 'auto' }}>
                {streams.length === 0 ? (
                  <p style={{ color: '#64748b', textAlign: 'center', padding: '1.5rem' }}>No demo streams yet — use "Create Stream" to see the UI walkthrough.</p>
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
                        <td style={{ padding: '0.75rem 0.5rem', color: '#fbbf24' }}>{s.status}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                )}
              </div>
            )}
          </section>

          {/* Right Column: Log */}
          <section style={{ background: '#090d16', padding: '1.5rem', borderRadius: '10px', border: '1px solid #1f2937', display: 'flex', flexDirection: 'column' }}>
            <h2 style={{ fontSize: '0.95rem', fontWeight: 600, color: '#94a3b8', margin: '0 0 1rem 0' }}>
              Demo Walkthrough Log
            </h2>

            <div style={{ background: '#030712', padding: '1.25rem', borderRadius: '8px', border: '1px solid #111827', fontFamily: 'Fira Code, monospace', fontSize: '0.8rem', color: '#10b981', flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '0.6rem' }}>
              {logs.map((log, idx) => (
                <div key={idx} style={{ color: '#fbbf24' }}>
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
