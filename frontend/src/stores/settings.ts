import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import apiClient from '../api/client';
import { exportData, importData } from '../api/data';
import type { UserConfig } from './config';

export const useSettingsStore = defineStore('settings', () => {
  // State
  const config = ref<UserConfig | null>(null);
  const originalConfig = ref<UserConfig | null>(null);
  const loading = ref(false);
  const saving = ref(false);
  const error = ref<string | null>(null);
  const saveSuccess = ref(false);

  // Gmail OAuth state machine
  const gmailStatus = ref<'disconnected' | 'connecting' | 'connected'>('disconnected');
  const deviceCode = ref<string | null>(null);
  const verificationUrl = ref<string | null>(null);
  const gmailPolling = ref(false);

  // Data management state
  const exporting = ref(false);
  const importing = ref(false);
  const importError = ref<string | null>(null);

  // Computed
  const hasChanges = computed(() => {
    if (!config.value || !originalConfig.value) return false;
    return JSON.stringify(config.value) !== JSON.stringify(originalConfig.value);
  });

  const notificationTier = computed<'gmail' | 'smtp' | 'desktop' | 'none'>(() => {
    if (!config.value) return 'none';
    if (config.value.gmail_refresh_token) return 'gmail';
    if (config.value.smtp_host && config.value.smtp_user) return 'smtp';
    return 'desktop';
  });

  // Actions
  const fetchConfig = async () => {
    loading.value = true;
    error.value = null;
    try {
      const response = await apiClient.get<UserConfig>('/api/config');
      config.value = response.data;
      originalConfig.value = JSON.parse(JSON.stringify(response.data));

      // Detect Gmail connection status
      if (response.data.gmail_refresh_token) {
        gmailStatus.value = 'connected';
      } else {
        gmailStatus.value = 'disconnected';
      }
    } catch (err: any) {
      error.value = err.response?.data?.detail || err.message || 'Failed to fetch settings';
    } finally {
      loading.value = false;
    }
  };

  const saveConfig = async () => {
    if (!config.value) return;
    saving.value = true;
    error.value = null;
    saveSuccess.value = false;
    try {
      await apiClient.post('/api/config', config.value);
      originalConfig.value = JSON.parse(JSON.stringify(config.value));
      saveSuccess.value = true;
      setTimeout(() => { saveSuccess.value = false; }, 3000);
    } catch (err: any) {
      error.value = err.response?.data?.detail || err.message || 'Failed to save settings';
    } finally {
      saving.value = false;
    }
  };

  const connectGmail = async () => {
    if (!config.value) return;
    gmailStatus.value = 'connecting';
    gmailPolling.value = true;
    deviceCode.value = null;
    verificationUrl.value = null;
    error.value = null;

    try {
      const response = await apiClient.post<{
        device_code: string;
        user_code: string;
        verification_url: string;
      }>('/api/gmail/connect', {
        client_id: config.value.gmail_client_id || '',
        client_secret: config.value.gmail_client_secret || '',
      });

      deviceCode.value = response.data.user_code;
      verificationUrl.value = response.data.verification_url;

      // Start polling loop
      pollGmailToken();
    } catch (err: any) {
      error.value = err.response?.data?.detail || err.message || 'Failed to initiate Gmail connection';
      gmailStatus.value = 'disconnected';
      gmailPolling.value = false;
    }
  };

  const pollGmailToken = async () => {
    if (!config.value || !deviceCode.value) return;

    const maxAttempts = 60; // 5 minutes with 5-second intervals
    let attempts = 0;

    const poll = async (): Promise<void> => {
      if (!gmailPolling.value || attempts >= maxAttempts) {
        if (gmailStatus.value === 'connecting') {
          gmailStatus.value = 'disconnected';
          error.value = 'Gmail authorization timed out. Please try again.';
        }
        gmailPolling.value = false;
        return;
      }

      attempts++;
      try {
        const response = await apiClient.post<{
          status: string;
          refresh_token?: string;
        }>('/api/gmail/token', {
          device_code: deviceCode.value,
          client_id: config.value!.gmail_client_id || '',
          client_secret: config.value!.gmail_client_secret || '',
        });

        if (response.data.status === 'success' && response.data.refresh_token) {
          // Update config with new refresh token
          config.value!.gmail_refresh_token = response.data.refresh_token;
          originalConfig.value = JSON.parse(JSON.stringify(config.value));
          gmailStatus.value = 'connected';
          gmailPolling.value = false;
          return;
        }
      } catch (err: any) {
        // Expected: "authorization_pending" means user hasn't entered code yet
        const detail = err.response?.data?.detail || '';
        if (detail.includes('authorization_pending') || detail.includes('slow_down')) {
          // Continue polling
        } else if (detail.includes('expired') || detail.includes('invalid')) {
          gmailStatus.value = 'disconnected';
          error.value = 'Gmail authorization expired or was denied. Please try again.';
          gmailPolling.value = false;
          return;
        }
      }

      // Wait 5 seconds before next poll
      await new Promise(resolve => setTimeout(resolve, 5000));
      return poll();
    };

    poll();
  };

  const cancelGmailPoll = () => {
    gmailPolling.value = false;
    gmailStatus.value = 'disconnected';
    deviceCode.value = null;
    verificationUrl.value = null;
  };

  const disconnectGmail = async () => {
    if (!config.value) return;
    try {
      await apiClient.post('/api/gmail/disconnect');
      config.value.gmail_refresh_token = undefined;
      delete (config.value as any).gmail_refresh_token;
      originalConfig.value = JSON.parse(JSON.stringify(config.value));
      gmailStatus.value = 'disconnected';
    } catch (err: any) {
      error.value = err.response?.data?.detail || err.message || 'Failed to disconnect Gmail';
    }
  };

  const doExportData = async () => {
    exporting.value = true;
    error.value = null;
    try {
      const blob = await exportData();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      const date = new Date().toISOString().split('T')[0];
      a.href = url;
      a.download = `coding-practice-export-${date}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (err: any) {
      error.value = err.response?.data?.detail || err.message || 'Failed to export data';
    } finally {
      exporting.value = false;
    }
  };

  const doImportData = async (file: File) => {
    importing.value = true;
    importError.value = null;
    try {
      await importData(file);
      // Refresh config after import
      await fetchConfig();
    } catch (err: any) {
      importError.value = err.response?.data?.detail || err.message || 'Failed to import data';
    } finally {
      importing.value = false;
    }
  };

  return {
    // State
    config,
    originalConfig,
    loading,
    saving,
    error,
    saveSuccess,
    gmailStatus,
    deviceCode,
    verificationUrl,
    gmailPolling,
    exporting,
    importing,
    importError,
    // Computed
    hasChanges,
    notificationTier,
    // Actions
    fetchConfig,
    saveConfig,
    connectGmail,
    cancelGmailPoll,
    disconnectGmail,
    doExportData,
    doImportData,
  };
});