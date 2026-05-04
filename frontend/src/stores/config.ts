import { defineStore } from 'pinia';
import { ref } from 'vue';
import apiClient from '../api/client';

export interface LanguageSkillLevel {
  language: string;
  skill_level: string;
}

export interface LanguageQuota {
  language: string;
  quota: number;
}

export interface UserConfig {
  preferred_language: string;
  skill_levels: LanguageSkillLevel[];
  daily_quotas: LanguageQuota[];
  ai_model: string;
  ai_model_type: string;
  email?: string;
  gmail_client_id?: string;
  gmail_client_secret?: string;
  smtp_host?: string;
  smtp_port?: number;
  smtp_user?: string;
  smtp_password?: string;
  reminder_time?: string;
  reminders_enabled?: boolean;
  gmail_refresh_token?: string;
  last_reminded_at?: string;
  sources_enabled?: string;   // JSON: {"github":true,"web":true,"ai_generated":true}
  source_priority?: string;   // JSON: ["github","web","ai_generated"]
  github_repos?: string;      // JSON: [{"url":"...","branch":"main"}]
  web_sources?: string;       // JSON: ["https://..."]
}

export const useConfigStore = defineStore('config', () => {
  // State
  const isConfigured = ref(false);
  const config = ref<UserConfig | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Actions
  const checkConfigured = async () => {
    loading.value = true;
    error.value = null;

    try {
      const response = await apiClient.get<{ configured: boolean }>('/api/config/check');
      isConfigured.value = response.data.configured;
      if (isConfigured.value) {
        await fetchConfig();
      }
    } catch (err: any) {
      if (err.response?.status === 404) {
        isConfigured.value = false;
      } else {
        error.value = err.message || 'Failed to check configuration status';
        throw err;
      }
    } finally {
      loading.value = false;
    }
  };

  const fetchConfig = async () => {
    loading.value = true;
    error.value = null;

    try {
      const response = await apiClient.get<UserConfig>('/api/config');
      config.value = response.data;
      isConfigured.value = true;
    } catch (err: any) {
      error.value = err.response?.data?.detail || err.message || 'Failed to fetch configuration';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const saveConfig = async (newConfig: UserConfig) => {
    loading.value = true;
    error.value = null;

    try {
      await apiClient.post('/api/config', newConfig);
      config.value = newConfig;
      isConfigured.value = true;
    } catch (err: any) {
      error.value = err.response?.data?.detail || err.message || 'Failed to save configuration';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const fetchOllamaModels = async () => {
    try {
      const response = await apiClient.get<Array<{ name: string; model_type: string }>>('/api/ollama/models');
      return response.data;
    } catch (err: any) {
      if (err.response?.status === 503) {
        // Ollama not available
        return [];
      }
      throw err;
    }
  };

  return {
    // State
    isConfigured,
    config,
    loading,
    error,
    // Actions
    checkConfigured,
    fetchConfig,
    saveConfig,
    fetchOllamaModels,
  };
});