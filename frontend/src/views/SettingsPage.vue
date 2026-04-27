<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useSettingsStore } from '../stores/settings';
import { useConfigStore } from '../stores/config';
import SectionNav from '../components/settings/SectionNav.vue';
import ToggleSwitch from '../components/settings/ToggleSwitch.vue';
import GmailConnectPanel from '../components/settings/GmailConnectPanel.vue';
import NotificationTierBadge from '../components/settings/NotificationTierBadge.vue';
import FileDropZone from '../components/settings/FileDropZone.vue';

const router = useRouter();
const settingsStore = useSettingsStore();
const configStore = useConfigStore();

const activeSection = ref('general');
const showImportConfirm = ref(false);
const importFile = ref<File | null>(null);
const importSuccess = ref(false);

const sections = [
  { id: 'general', label: 'General', icon: '<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>' },
  { id: 'ai-model', label: 'AI Model', icon: '<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2a4 4 0 0 0-4 4v2H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V10a2 2 0 0 0-2-2h-2V6a4 4 0 0 0-4-4z"/></svg>' },
  { id: 'email-notifications', label: 'Email & Notifications', icon: '<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 4L12 13 2 4"/></svg>' },
  { id: 'gmail', label: 'Gmail Connection', icon: '<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 4L12 13 2 4"/></svg>' },
  { id: 'data-management', label: 'Data Management', icon: '<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>' },
];

const languageOptions = [
  { value: 'python', label: 'Python' },
  { value: 'rust', label: 'Rust' },
  { value: 'go', label: 'Go' },
  { value: 'cpp', label: 'C++' },
];

const skillLevelOptions = [
  { value: 'beginner', label: 'Beginner' },
  { value: 'intermediate', label: 'Intermediate' },
  { value: 'advanced', label: 'Advanced' },
];

// Ollama models
const ollamaModels = ref<Array<{ name: string }>>([]);
const ollamaLoading = ref(false);

const apiModelOptions = [
  { value: 'gpt-4o', label: 'gpt-4o' },
  { value: 'gpt-4o-mini', label: 'gpt-4o-mini' },
  { value: 'claude-3-5-sonnet', label: 'claude-3-5-sonnet' },
  { value: 'claude-3-5-haiku', label: 'claude-3-5-haiku' },
];

const modelTypeOptions = [
  { value: 'local', label: 'Local (Ollama)' },
  { value: 'api', label: 'API' },
];

const availableModels = computed(() => {
  if (settingsStore.config?.ai_model_type === 'local') {
    return ollamaModels.value.map(m => ({ value: m.name, label: m.name }));
  }
  return apiModelOptions;
});

onMounted(async () => {
  await settingsStore.fetchConfig();

  // Load Ollama models for AI model section
  ollamaLoading.value = true;
  try {
    ollamaModels.value = await configStore.fetchOllamaModels();
  } catch {
    // Ollama may not be available
  } finally {
    ollamaLoading.value = false;
  }

  // Intersection observer for scroll-to-section highlight
  setupScrollObserver();
});

onUnmounted(() => {
  if (scrollObserver) scrollObserver.disconnect();
});

// Scroll-to-section logic
let scrollObserver: IntersectionObserver | null = null;

const scrollToSection = (sectionId: string) => {
  activeSection.value = sectionId;
  const el = document.getElementById(sectionId);
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }
};

const setupScrollObserver = () => {
  scrollObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          activeSection.value = entry.target.id;
        }
      }
    },
    { rootMargin: '-20% 0px -70% 0px' }
  );

  sections.forEach(s => {
    const el = document.getElementById(s.id);
    if (el) scrollObserver!.observe(el);
  });
};

// Navigation
const goBack = () => {
  router.push({ name: 'dashboard' });
};

// Save
const handleSave = async () => {
  await settingsStore.saveConfig();
};

// Import
const onImportFileSelected = (file: File) => {
  importFile.value = file;
  showImportConfirm.value = true;
};

const confirmImport = async () => {
  if (!importFile.value) return;
  showImportConfirm.value = false;
  await settingsStore.doImportData(importFile.value);
  if (!settingsStore.importError) {
    importSuccess.value = true;
    importFile.value = null;
    setTimeout(() => { importSuccess.value = false; }, 3000);
  }
};

const cancelImport = () => {
  showImportConfirm.value = false;
  importFile.value = null;
};

// Skill level per language
const getSkillLevelForLanguage = (lang: string): string => {
  return settingsStore.config?.skill_levels?.find(s => s.language === lang)?.skill_level || 'beginner';
};

const setSkillLevelForLanguage = (lang: string, level: string) => {
  if (!settingsStore.config) return;
  const existing = settingsStore.config.skill_levels.find(s => s.language === lang);
  if (existing) {
    existing.skill_level = level;
  } else {
    settingsStore.config.skill_levels.push({ language: lang, skill_level: level });
  }
};

const getQuotaForLanguage = (lang: string): number => {
  return settingsStore.config?.daily_quotas?.find(q => q.language === lang)?.quota || 1;
};

const setQuotaForLanguage = (lang: string, quota: number) => {
  if (!settingsStore.config) return;
  const existing = settingsStore.config.daily_quotas.find(q => q.language === lang);
  if (existing) {
    existing.quota = Math.max(1, quota);
  } else {
    settingsStore.config.daily_quotas.push({ language: lang, quota: Math.max(1, quota) });
  }
};

const removeLanguage = (lang: string) => {
  if (!settingsStore.config) return;
  settingsStore.config.skill_levels = settingsStore.config.skill_levels.filter(s => s.language !== lang);
  settingsStore.config.daily_quotas = settingsStore.config.daily_quotas.filter(q => q.language !== lang);
};

const addLanguage = (lang: string) => {
  if (!settingsStore.config) return;
  if (settingsStore.config.skill_levels.some(s => s.language === lang)) return;
  settingsStore.config.skill_levels.push({ language: lang, skill_level: 'beginner' });
  settingsStore.config.daily_quotas.push({ language: lang, quota: 1 });
  if (!settingsStore.config.preferred_language) {
    settingsStore.config.preferred_language = lang;
  }
};

const languagesNotYetAdded = computed(() => {
  const added = new Set(settingsStore.config?.skill_levels?.map(s => s.language) || []);
  return languageOptions.filter(l => !added.has(l.value));
});
</script>

<template>
  <div class="settings-page">
    <!-- Header -->
    <header class="settings-header">
      <div class="header-left">
        <button @click="goBack" class="back-btn" title="Back to Dashboard">
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
        </button>
        <h1 class="header-title">Settings</h1>
      </div>
    </header>

    <!-- Loading State -->
    <div v-if="settingsStore.loading && !settingsStore.config" class="loading-state">
      <div class="loading-spinner"></div>
      <p>Loading settings...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="settingsStore.error && !settingsStore.config" class="error-state">
      <p>{{ settingsStore.error }}</p>
      <button @click="settingsStore.fetchConfig" class="retry-btn">Retry</button>
    </div>

    <!-- Main Content -->
    <div v-else-if="settingsStore.config" class="settings-layout">
      <!-- Side Panel -->
      <aside class="side-panel">
        <nav class="side-nav">
          <SectionNav
            v-for="section in sections"
            :key="section.id"
            :label="section.label"
            :section-id="section.id"
            :icon="section.icon"
            :active="activeSection === section.id"
            @navigate="scrollToSection"
          />
        </nav>
      </aside>

      <!-- Main Content Area -->
      <main class="settings-main">
        <!-- General Section -->
        <section id="general" class="settings-section">
          <h2 class="section-heading">
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
            General
          </h2>
          <div class="section-body">
            <!-- Preferred Language -->
            <div class="form-group">
              <label class="form-label" for="preferred-language">Preferred Language</label>
              <select
                id="preferred-language"
                v-model="settingsStore.config.preferred_language"
                class="form-select"
              >
                <option v-for="lang in languageOptions" :key="lang.value" :value="lang.value">
                  {{ lang.label }}
                </option>
              </select>
            </div>

            <!-- Per-Language Config -->
            <div class="language-configs">
              <h3 class="sub-heading">Languages</h3>
              <div
                v-for="skillLevel in settingsStore.config.skill_levels"
                :key="skillLevel.language"
                class="language-config-row"
              >
                <div class="lang-config-header">
                  <span class="lang-name">{{ languageOptions.find(l => l.value === skillLevel.language)?.label || skillLevel.language }}</span>
                  <button
                    v-if="settingsStore.config.skill_levels.length > 1"
                    @click="removeLanguage(skillLevel.language)"
                    class="remove-lang-btn"
                    title="Remove language"
                  >
                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
                  </button>
                </div>
                <div class="lang-config-fields">
                  <div class="form-group compact">
                    <label class="form-label" :for="`skill-${skillLevel.language}`">Skill Level</label>
                    <select
                      :id="`skill-${skillLevel.language}`"
                      :value="getSkillLevelForLanguage(skillLevel.language)"
                      @change="setSkillLevelForLanguage(skillLevel.language, ($event.target as HTMLSelectElement).value)"
                      class="form-select"
                    >
                      <option v-for="opt in skillLevelOptions" :key="opt.value" :value="opt.value">
                        {{ opt.label }}
                      </option>
                    </select>
                  </div>
                  <div class="form-group compact">
                    <label class="form-label" :for="`quota-${skillLevel.language}`">Daily Quota</label>
                    <input
                      :id="`quota-${skillLevel.language}`"
                      type="number"
                      min="1"
                      max="20"
                      :value="getQuotaForLanguage(skillLevel.language)"
                      @input="setQuotaForLanguage(skillLevel.language, ($event.target as HTMLInputElement).valueAsNumber)"
                      class="form-input w-24"
                    />
                  </div>
                </div>
              </div>

              <!-- Add Language -->
              <div v-if="languagesNotYetAdded.length > 0" class="add-language">
                <select
                  @change="addLanguage(($event.target as HTMLSelectElement).value); ($event.target as HTMLSelectElement).value = ''"
                  class="form-select"
                >
                  <option value="">Add a language...</option>
                  <option v-for="lang in languagesNotYetAdded" :key="lang.value" :value="lang.value">
                    {{ lang.label }}
                  </option>
                </select>
              </div>
            </div>
          </div>
        </section>

        <!-- AI Model Section -->
        <section id="ai-model" class="settings-section">
          <h2 class="section-heading">
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2a4 4 0 0 0-4 4v2H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V10a2 2 0 0 0-2-2h-2V6a4 4 0 0 0-4-4z"/></svg>
            AI Model
          </h2>
          <div class="section-body">
            <!-- Model Type -->
            <div class="form-group">
              <label class="form-label">Model Type</label>
              <div class="radio-group">
                <label v-for="opt in modelTypeOptions" :key="opt.value" class="radio-label">
                  <input
                    type="radio"
                    :value="opt.value"
                    v-model="settingsStore.config.ai_model_type"
                    class="radio-input"
                  />
                  <span class="radio-text">{{ opt.label }}</span>
                </label>
              </div>
            </div>

            <!-- Model Selection (Local) -->
            <div v-if="settingsStore.config.ai_model_type === 'local'" class="form-group">
              <label class="form-label" for="ollama-model">Model</label>
              <select
                id="ollama-model"
                v-model="settingsStore.config.ai_model"
                class="form-select"
              >
                <option v-if="ollamaLoading" disabled value="">Loading models...</option>
                <option v-for="model in ollamaModels" :key="model.name" :value="model.name">
                  {{ model.name }}
                </option>
              </select>
              <p v-if="!ollamaLoading && ollamaModels.length === 0" class="hint-text">
                No Ollama models found. Make sure Ollama is running.
              </p>
            </div>

            <!-- Model Input (API) -->
            <div v-if="settingsStore.config.ai_model_type === 'api'" class="form-group">
              <label class="form-label" for="api-model">Model Name</label>
              <select
                id="api-model"
                v-model="settingsStore.config.ai_model"
                class="form-select"
              >
                <option v-for="opt in apiModelOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
            </div>
          </div>
        </section>

        <!-- Email & Notifications Section -->
        <section id="email-notifications" class="settings-section">
          <h2 class="section-heading">
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 4L12 13 2 4"/></svg>
            Email & Notifications
          </h2>
          <div class="section-body">
            <!-- Enable Reminders Toggle -->
            <div class="form-group">
              <div class="flex items-center justify-between">
                <label class="form-label" for="reminders-enabled">Enable Reminders</label>
                <ToggleSwitch
                  v-model="settingsStore.config.reminders_enabled"
                  id="reminders-enabled"
                />
              </div>
            </div>

            <!-- Notification Tier -->
            <div class="form-group">
              <NotificationTierBadge :tier="settingsStore.notificationTier" />
            </div>

            <!-- Email -->
            <div class="form-group">
              <label class="form-label" for="email">Email Address</label>
              <input
                id="email"
                type="email"
                v-model="settingsStore.config.email"
                placeholder="your@email.com"
                class="form-input"
              />
            </div>

            <!-- Reminder Time -->
            <div class="form-group">
              <label class="form-label" for="reminder-time">Reminder Time</label>
              <input
                id="reminder-time"
                type="time"
                v-model="settingsStore.config.reminder_time"
                class="form-input w-40"
              />
            </div>

            <hr class="section-divider" />

            <h3 class="sub-heading">SMTP Configuration</h3>

            <!-- SMTP Host -->
            <div class="form-group">
              <label class="form-label" for="smtp-host">SMTP Host</label>
              <input
                id="smtp-host"
                type="text"
                v-model="settingsStore.config.smtp_host"
                placeholder="smtp.gmail.com"
                class="form-input"
              />
            </div>

            <!-- SMTP Port -->
            <div class="form-group">
              <label class="form-label" for="smtp-port">SMTP Port</label>
              <input
                id="smtp-port"
                type="number"
                v-model.number="settingsStore.config.smtp_port"
                placeholder="587"
                class="form-input w-32"
              />
            </div>

            <!-- SMTP User -->
            <div class="form-group">
              <label class="form-label" for="smtp-user">SMTP Username</label>
              <input
                id="smtp-user"
                type="text"
                v-model="settingsStore.config.smtp_user"
                placeholder="your@email.com"
                class="form-input"
              />
            </div>

            <!-- SMTP Password -->
            <div class="form-group">
              <label class="form-label" for="smtp-password">SMTP Password</label>
              <input
                id="smtp-password"
                type="password"
                v-model="settingsStore.config.smtp_password"
                placeholder="App-specific password"
                class="form-input"
              />
            </div>

            <hr class="section-divider" />

            <h3 class="sub-heading">Gmail OAuth Credentials</h3>
            <p class="hint-text">These are only needed for Gmail API notification. The Connect button below handles the OAuth flow.</p>

            <!-- Gmail Client ID -->
            <div class="form-group">
              <label class="form-label" for="gmail-client-id">Gmail Client ID</label>
              <input
                id="gmail-client-id"
                type="text"
                v-model="settingsStore.config.gmail_client_id"
                placeholder="your-client-id.apps.googleusercontent.com"
                class="form-input"
              />
            </div>

            <!-- Gmail Client Secret -->
            <div class="form-group">
              <label class="form-label" for="gmail-client-secret">Gmail Client Secret</label>
              <input
                id="gmail-client-secret"
                type="password"
                v-model="settingsStore.config.gmail_client_secret"
                placeholder="GOCSPX-..."
                class="form-input"
              />
            </div>
          </div>
        </section>

        <!-- Gmail Connection Section -->
        <section id="gmail" class="settings-section">
          <h2 class="section-heading">
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/><polyline points="10 17 15 12 10 7"/><line x1="15" y1="12" x2="3" y2="12"/></svg>
            Gmail Connection
          </h2>
          <div class="section-body">
            <p class="section-description">
              Connect your Gmail account to send reminders via the Gmail API. This uses a device code flow
              that does not require a redirect URI.
            </p>
            <GmailConnectPanel />
          </div>
        </section>

        <!-- Data Management Section -->
        <section id="data-management" class="settings-section">
          <h2 class="section-heading">
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
            Data Management
          </h2>
          <div class="section-body">
            <!-- Export -->
            <div class="data-action">
              <h3 class="sub-heading">Export Data</h3>
              <p class="hint-text">Download all your data as a JSON file. This includes your configuration, exercises, submissions, and schedule.</p>
              <button
                @click="settingsStore.doExportData"
                :disabled="settingsStore.exporting"
                class="action-btn primary"
              >
                <span v-if="settingsStore.exporting" class="btn-spinner"></span>
                {{ settingsStore.exporting ? 'Generating...' : 'Export All Data' }}
              </button>
            </div>

            <hr class="section-divider" />

            <!-- Import -->
            <div class="data-action">
              <h3 class="sub-heading">Import Data</h3>
              <div class="import-warning">
                <svg class="w-5 h-5 text-amber-500 flex-shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
                <span>Importing will <strong>overwrite all existing data</strong>. This cannot be undone.</span>
              </div>
              <FileDropZone @file-selected="onImportFileSelected" :disabled="settingsStore.importing" />
              <div v-if="settingsStore.importing" class="import-progress">
                <div class="btn-spinner"></div>
                <span>Importing data...</span>
              </div>
              <div v-if="importSuccess" class="import-success">
                Data imported successfully. Refreshing...
              </div>
              <div v-if="settingsStore.importError" class="import-error">
                {{ settingsStore.importError }}
              </div>
            </div>
          </div>
        </section>
      </main>
    </div>

    <!-- Save Bar -->
    <div v-if="settingsStore.config" class="save-bar">
      <div class="save-bar-inner">
        <!-- Error -->
        <div v-if="settingsStore.error" class="save-error">
          {{ settingsStore.error }}
        </div>

        <!-- Success -->
        <div v-if="settingsStore.saveSuccess" class="save-success">
          Settings saved
        </div>

        <div class="save-bar-right">
          <button
            @click="goBack"
            class="action-btn secondary"
          >
            Cancel
          </button>
          <button
            @click="handleSave"
            :disabled="!settingsStore.hasChanges || settingsStore.saving"
            class="action-btn primary"
          >
            <span v-if="settingsStore.saving" class="btn-spinner"></span>
            {{ settingsStore.saving ? 'Saving...' : 'Save Changes' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Import Confirmation Modal -->
    <div v-if="showImportConfirm" class="modal-overlay" @click.self="cancelImport">
      <div class="modal-content">
        <h3 class="modal-title">Confirm Import</h3>
        <p class="modal-text">
          This will <strong>replace all existing data</strong> with the contents of
          <code>{{ importFile?.name }}</code>. This action cannot be undone.
        </p>
        <div class="modal-actions">
          <button @click="cancelImport" class="action-btn secondary">Cancel</button>
          <button @click="confirmImport" class="action-btn danger">Import Data</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  min-height: 100vh;
  background-color: #f3f4f6;
}

/* Header */
.settings-header {
  background-color: white;
  border-bottom: 1px solid #e5e7eb;
  padding: 1rem 1.5rem;
  position: sticky;
  top: 0;
  z-index: 10;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.back-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: 0.5rem;
  border: none;
  background: none;
  cursor: pointer;
  color: #6b7280;
  transition: all 0.2s;
}

.back-btn:hover {
  background-color: #f3f4f6;
  color: #1f2937;
}

.header-title {
  font-size: 1.25rem;
  font-weight: 700;
  color: #1f2937;
  margin: 0;
}

/* Layout */
.settings-layout {
  display: flex;
  max-width: 80rem;
  margin: 0 auto;
  padding-bottom: 5rem;
}

/* Side Panel */
.side-panel {
  width: 200px;
  min-width: 200px;
  background-color: #f9fafb;
  border-right: 1px solid #e5e7eb;
  min-height: calc(100vh - 4rem);
  position: sticky;
  top: 4rem;
  padding-top: 1rem;
}

.side-nav {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 0 0.5rem;
}

/* Main Content */
.settings-main {
  flex: 1;
  padding: 1.5rem 2rem;
  max-width: 50rem;
}

/* Sections */
.settings-section {
  margin-bottom: 2rem;
  scroll-margin-top: 5rem;
}

.section-heading {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1.25rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0 0 1rem 0;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid #e5e7eb;
}

.section-body {
  padding-left: 0.25rem;
}

.section-description {
  color: #6b7280;
  font-size: 0.9rem;
  margin-bottom: 1rem;
  line-height: 1.5;
}

.section-divider {
  border: none;
  border-top: 1px solid #e5e7eb;
  margin: 1.5rem 0;
}

.sub-heading {
  font-size: 0.875rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0 0 0.75rem 0;
}

/* Form Elements */
.form-group {
  margin-bottom: 1rem;
}

.form-group.compact {
  margin-bottom: 0;
}

.form-label {
  display: block;
  font-size: 0.875rem;
  font-weight: 600;
  color: #374151;
  margin-bottom: 0.375rem;
}

.form-input,
.form-select {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: 1px solid #d1d5db;
  border-radius: 0.5rem;
  font-size: 0.875rem;
  color: #1f2937;
  background-color: white;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.radio-group {
  display: flex;
  gap: 1.5rem;
  margin-top: 0.375rem;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
  color: #374151;
}

.radio-input {
  accent-color: #3b82f6;
}

.hint-text {
  font-size: 0.8rem;
  color: #9ca3af;
  margin-top: 0.25rem;
}

/* Language Configs */
.language-configs {
  margin-top: 1rem;
}

.language-config-row {
  background-color: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  padding: 1rem;
  margin-bottom: 0.75rem;
}

.lang-config-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}

.lang-name {
  font-weight: 600;
  color: #1f2937;
}

.remove-lang-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 0.25rem;
  border: none;
  background: none;
  cursor: pointer;
  color: #9ca3af;
  transition: all 0.2s;
}

.remove-lang-btn:hover {
  background-color: #fef2f2;
  color: #dc2626;
}

.lang-config-fields {
  display: flex;
  gap: 1rem;
  align-items: flex-end;
}

.add-language {
  margin-top: 0.75rem;
}

/* Data Management */
.data-action {
  margin-bottom: 0.5rem;
}

.import-warning {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  background-color: #fffbeb;
  border: 1px solid #fde68a;
  border-radius: 0.5rem;
  padding: 0.75rem 1rem;
  margin-bottom: 1rem;
  font-size: 0.85rem;
  color: #92400e;
}

.import-progress,
.import-success,
.import-error {
  margin-top: 0.75rem;
  font-size: 0.85rem;
}

.import-progress {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: #3b82f6;
}

.import-success {
  color: #16a34a;
  background-color: #f0fdf4;
  border: 1px solid #bbf7d0;
  border-radius: 0.5rem;
  padding: 0.5rem 0.75rem;
}

.import-error {
  color: #dc2626;
  background-color: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 0.5rem;
  padding: 0.5rem 0.75rem;
}

/* Buttons */
.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  font-size: 0.875rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.action-btn.primary {
  background-color: #3b82f6;
  color: white;
  border-color: #3b82f6;
}

.action-btn.primary:hover:not(:disabled) {
  background-color: #2563eb;
}

.action-btn.primary:disabled {
  background-color: #9ca3af;
  cursor: not-allowed;
  border-color: #9ca3af;
}

.action-btn.secondary {
  background-color: white;
  color: #374151;
  border-color: #d1d5db;
}

.action-btn.secondary:hover {
  background-color: #f9fafb;
}

.action-btn.danger {
  background-color: #dc2626;
  color: white;
  border-color: #dc2626;
}

.action-btn.danger:hover {
  background-color: #b91c1c;
}

.btn-spinner {
  display: inline-block;
  width: 1rem;
  height: 1rem;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

/* Save Bar */
.save-bar {
  position: sticky;
  bottom: 0;
  background-color: white;
  border-top: 1px solid #e5e7eb;
  padding: 0.75rem 1.5rem;
  z-index: 10;
}

.save-bar-inner {
  max-width: 80rem;
  margin: 0 auto;
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 1rem;
}

.save-bar-right {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.save-error {
  color: #dc2626;
  font-size: 0.85rem;
  margin-right: auto;
}

.save-success {
  color: #16a34a;
  font-size: 0.85rem;
  font-weight: 600;
  margin-right: auto;
}

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
}

.modal-content {
  background-color: white;
  border-radius: 0.75rem;
  padding: 1.5rem;
  max-width: 28rem;
  width: 90%;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
}

.modal-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0 0 0.75rem 0;
}

.modal-text {
  font-size: 0.875rem;
  color: #374151;
  line-height: 1.5;
  margin-bottom: 1.25rem;
}

.modal-text code {
  background-color: #f3f4f6;
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-size: 0.8rem;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
}

/* Loading & Error */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem;
  color: #6b7280;
  gap: 1rem;
}

.loading-spinner {
  width: 2.5rem;
  height: 2.5rem;
  border: 3px solid #e5e7eb;
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.error-state {
  text-align: center;
  padding: 3rem;
  color: #dc2626;
}

.retry-btn {
  margin-top: 1rem;
  padding: 0.5rem 1rem;
  background-color: #3b82f6;
  color: white;
  border: none;
  border-radius: 0.5rem;
  cursor: pointer;
  font-weight: 600;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Responsive */
@media (max-width: 768px) {
  .settings-layout {
    flex-direction: column;
  }

  .side-panel {
    width: 100%;
    min-width: 100%;
    min-height: auto;
    position: static;
    border-right: none;
    border-bottom: 1px solid #e5e7eb;
  }

  .side-nav {
    flex-direction: row;
    overflow-x: auto;
    padding: 0.5rem;
    gap: 0.5rem;
  }

  .settings-main {
    padding: 1rem;
  }
}
</style>