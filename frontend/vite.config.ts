import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

// @stellar/stellar-sdk relies on Node globals (Buffer, etc.) that browsers don't provide —
// this polyfills them for real, not a workaround around a problem that doesn't exist.
export default defineConfig({
  plugins: [react(), nodePolyfills({ globals: { Buffer: true, global: true, process: true } })],
  server: {
    port: 5173,
    host: true
  }
});
