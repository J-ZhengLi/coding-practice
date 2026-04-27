<script setup lang="ts">
import { useSettingsStore } from '../../stores/settings';

const settingsStore = useSettingsStore();

const copyDeviceCode = async () => {
  if (settingsStore.deviceCode) {
    try {
      await navigator.clipboard.writeText(settingsStore.deviceCode);
    } catch {
      // Fallback: select text for manual copy
    }
  }
};
</script>

<template>
  <div class="gmail-connect-panel">
    <!-- Disconnected State -->
    <div v-if="settingsStore.gmailStatus === 'disconnected'" class="gmail-state gmail-disconnected">
      <div class="state-icon">
        <svg class="w-8 h-8 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 4L12 13 2 4"/></svg>
      </div>
      <p class="state-text text-gray-600">Not connected to Gmail</p>
      <button
        @click="settingsStore.connectGmail"
        class="connect-btn"
      >
        Connect Gmail
      </button>
      <p class="hint-text">Uses device code flow — no redirect URI needed for this local app.</p>
    </div>

    <!-- Connecting State -->
    <div v-if="settingsStore.gmailStatus === 'connecting'" class="gmail-state gmail-connecting">
      <div class="state-icon">
        <div class="polling-spinner"></div>
      </div>
      <p class="state-text font-medium text-blue-700">Waiting for authorization...</p>

      <div v-if="settingsStore.deviceCode" class="device-code-section">
        <p class="text-sm text-gray-600 mb-1">Your code:</p>
        <div class="device-code-display">
          <code class="device-code">{{ settingsStore.deviceCode }}</code>
          <button @click="copyDeviceCode" class="copy-btn" title="Copy code">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
          </button>
        </div>
      </div>

      <div v-if="settingsStore.verificationUrl" class="verification-url-section">
        <p class="text-sm text-gray-600 mb-1">Visit this URL and enter the code above:</p>
        <a
          :href="settingsStore.verificationUrl"
          target="_blank"
          rel="noopener noreferrer"
          class="verification-link"
        >
          {{ settingsStore.verificationUrl }}
        </a>
      </div>

      <button @click="settingsStore.cancelGmailPoll" class="cancel-btn">
        Cancel
      </button>
    </div>

    <!-- Connected State -->
    <div v-if="settingsStore.gmailStatus === 'connected'" class="gmail-state gmail-connected">
      <div class="state-icon">
        <svg class="w-8 h-8 text-green-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
      </div>
      <p class="state-text font-medium text-green-700">Connected to Gmail</p>
      <button @click="settingsStore.disconnectGmail" class="disconnect-btn">
        Disconnect
      </button>
    </div>
  </div>
</template>

<style scoped>
.gmail-connect-panel {
  background-color: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 0.75rem;
  padding: 1.5rem;
}

.gmail-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
  text-align: center;
}

.state-icon {
  margin-bottom: 0.25rem;
}

.state-text {
  font-size: 1rem;
}

.connect-btn {
  background-color: #3b82f6;
  color: white;
  border: none;
  border-radius: 0.5rem;
  padding: 0.625rem 1.5rem;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
}

.connect-btn:hover {
  background-color: #2563eb;
}

.hint-text {
  font-size: 0.75rem;
  color: #9ca3af;
  margin-top: 0.5rem;
}

.device-code-section,
.verification-url-section {
  width: 100%;
  text-align: center;
}

.device-code-display {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  background-color: white;
  border: 2px solid #3b82f6;
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
}

.device-code {
  font-size: 1.25rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  color: #1d4ed8;
}

.copy-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: #6b7280;
  padding: 0.25rem;
  transition: color 0.2s;
}

.copy-btn:hover {
  color: #3b82f6;
}

.verification-link {
  color: #3b82f6;
  text-decoration: underline;
  font-size: 0.9rem;
  word-break: break-all;
}

.verification-link:hover {
  color: #2563eb;
}

.cancel-btn {
  background-color: #f3f4f6;
  color: #374151;
  border: 1px solid #d1d5db;
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
  font-size: 0.85rem;
  cursor: pointer;
  transition: background-color 0.2s;
}

.cancel-btn:hover {
  background-color: #e5e7eb;
}

.disconnect-btn {
  background-color: white;
  color: #dc2626;
  border: 1px solid #fca5a5;
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.disconnect-btn:hover {
  background-color: #fef2f2;
  border-color: #dc2626;
}

.polling-spinner {
  width: 2rem;
  height: 2rem;
  border: 3px solid #dbeafe;
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>