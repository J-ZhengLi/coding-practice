<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useSettingsStore } from '../stores/settings';
import { useConfigStore } from '../stores/config';
import { sendTestReminder } from '../api/reminder';
import SectionNav from '../components/settings/SectionNav.vue';
import ToggleSwitch from '../components/settings/ToggleSwitch.vue';
import GmailConnectPanel from '../components/settings/GmailConnectPanel.vue';
import FileDropZone from '../components/settings/FileDropZone.vue';
import SourceCard from '../components/settings/SourceCard.vue';

const router = useRouter();
const settingsStore = useSettingsStore();
const configStore = useConfigStore();

const activeSection = ref('general');
const showImportConfirm = ref(false);
const importFile = ref<File | null>(null);
const importSuccess = ref(false);

// --- Exercise Sources State ---
const sourceLabels: Record<string, string> = {
  github: 'GitHub',
  web: 'Web',
  ai_generated: 'AI-Generated',
};

const sourcePriority = ref<string[]>(['github', 'web', 'ai_generated']);
const sourcesEnabled = ref<Record<string, boolean>>({
  github: true,
  web: true,
  ai_generated: true,
});
const githubRepos = ref<Array<{ url: string; branch: string }>>([]);
const webSources = ref<string[]>([]);

const enabledCount = computed(() =>
  Object.values(sourcesEnabled.value).filter(Boolean).length
);

// New repo/URL input state for add forms
const newGithubRepoUrl = ref('');
const newGithubRepoBranch = ref('');
const newWebSourceUrl = ref('');
const urlValidationError = ref<Record<string, string>>({});

const sections = [
  { id: 'general', label: 'General', icon: '<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>' },
  { id: 'ai-model', label: 'AI Model', icon: '<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2a4 4 0 0 0-4 4v2H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V10a2 2 0 0 0-2-2h-2V6a4 4 0 0 0-4-4z"/></svg>' },
  { id: 'exercise-sources', label: 'Exercise Sources', icon: '<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 6h16M4 12h16M4 18h16"/><circle cx="8" cy="6" r="1.5" fill="currentColor"/><circle cx="8" cy="12" r="1.5" fill="currentColor"/><circle cx="8" cy="18" r="1.5" fill="currentColor"/></svg>' },
  { id: 'notifications', label: 'Notifications', icon: '<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>' },
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

const modelTypeOptions = [
  { value: 'local', label: 'Local (Ollama)' },
  { value: 'api', label: 'API' },
];

const reminderMethodOptions = [
  { value: 'gmail', label: 'Gmail' },
  { value: 'smtp', label: 'SMTP' },
  { value: 'desktop', label: 'Desktop notification' },
];

const reminderMethod = computed<'gmail' | 'smtp' | 'desktop'>({
  get: () => {
    if (!settingsStore.config) return 'desktop';
    if (settingsStore.config.gmail_refresh_token) return 'gmail';
    if (settingsStore.config.smtp_host) return 'smtp';
    return 'desktop';
  },
  set: (value: 'gmail' | 'smtp' | 'desktop') => {
    if (!settingsStore.config) return;
    // Clear fields from the previous method
    settingsStore.config.smtp_host = '';
    settingsStore.config.smtp_port = undefined;
    settingsStore.config.smtp_user = '';
    settingsStore.config.smtp_password = '';
    settingsStore.config.gmail_client_id = '';
    settingsStore.config.gmail_client_secret = '';
  },
});

// --- Exercise Sources Logic ---
const initSourceConfig = () => {
  if (!settingsStore.config) return;
  // Parse source_priority JSON string
  if (settingsStore.config.source_priority) {
    try {
      const parsed = JSON.parse(settingsStore.config.source_priority);
      if (Array.isArray(parsed) && parsed.length > 0) {
        sourcePriority.value = parsed;
      }
    } catch { /* keep defaults */ }
  }
  // Parse sources_enabled JSON string
  if (settingsStore.config.sources_enabled) {
    try {
      const parsed = JSON.parse(settingsStore.config.sources_enabled);
      if (typeof parsed === 'object' && parsed !== null) {
        sourcesEnabled.value = {
          github: parsed.github ?? true,
          web: parsed.web ?? true,
          ai_generated: parsed.ai_generated ?? true,
        };
      }
    } catch { /* keep defaults */ }
  }
  // Parse github_repos JSON string
  if (settingsStore.config.github_repos) {
    try {
      const parsed = JSON.parse(settingsStore.config.github_repos);
      if (Array.isArray(parsed)) {
        githubRepos.value = parsed.map((r: any) => ({
          url: r.url || '',
          branch: r.branch || '',
        }));
      }
    } catch { /* keep defaults */ }
  }
  // Parse web_sources JSON string
  if (settingsStore.config.web_sources) {
    try {
      const parsed = JSON.parse(settingsStore.config.web_sources);
      if (Array.isArray(parsed)) {
        webSources.value = parsed;
      }
    } catch { /* keep defaults */ }
  }
  // If no web_sources in config, pre-fill CURATED_SITES per D-07
  if (webSources.value.length === 0 && !settingsStore.config.web_sources) {
    webSources.value = [
      'https://doc.rust-lang.org/rust-by-example/',
      'https://gobyexample.com/',
      'https://docs.python.org/3/tutorial/',
      'https://en.cppreference.com/w/',
    ];
  }
};

// Toggle a source on/off
const onSourceToggle = (sourceKey: string, enabled: boolean) => {
  sourcesEnabled.value[sourceKey] = enabled;
  settingsStore.config.sources_enabled = JSON.stringify(sourcesEnabled.value);
};

// --- Drag and Drop ---
let draggedIndex: number | null = null;

const onDragStart = (event: DragEvent, index: number) => {
  draggedIndex = index;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
  }
  (event.target as HTMLElement)?.closest('.source-card')?.classList.add('opacity-50');
};

const onDragOver = (event: DragEvent) => {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
};

const onDrop = (event: DragEvent, dropIndex: number) => {
  event.preventDefault();
  document.querySelectorAll('.source-card.opacity-50').forEach(el => el.classList.remove('opacity-50'));

  if (draggedIndex === null || draggedIndex === dropIndex) {
    draggedIndex = null;
    return;
  }

  const items = [...sourcePriority.value];
  const [moved] = items.splice(draggedIndex, 1);
  items.splice(dropIndex, 0, moved);
  sourcePriority.value = items;
  draggedIndex = null;

  settingsStore.config.source_priority = JSON.stringify(sourcePriority.value);
};

// --- URL Validation ---
const isValidHttpUrl = (url: string): boolean => {
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'http:' || parsed.protocol === 'https:';
  } catch {
    return false;
  }
};

// --- GitHub Repo Management ---
const addGithubRepo = () => {
  const url = newGithubRepoUrl.value.trim();
  if (!url) return;
  if (!isValidHttpUrl(url)) {
    urlValidationError.value['github-repo'] = 'Please enter a valid URL starting with http:// or https://';
    return;
  }
  urlValidationError.value['github-repo'] = '';
  githubRepos.value.push({
    url,
    branch: newGithubRepoBranch.value.trim(),
  });
  newGithubRepoUrl.value = '';
  newGithubRepoBranch.value = '';
  settingsStore.config.github_repos = JSON.stringify(githubRepos.value);
};

const removeGithubRepo = (index: number) => {
  githubRepos.value.splice(index, 1);
  settingsStore.config.github_repos = JSON.stringify(githubRepos.value);
};

// --- Web Source Management ---
const addWebSource = () => {
  const url = newWebSourceUrl.value.trim();
  if (!url) return;
  if (!isValidHttpUrl(url)) {
    urlValidationError.value['web-source'] = 'Please enter a valid URL starting with http:// or https://';
    return;
  }
  urlValidationError.value['web-source'] = '';
  webSources.value.push(url);
  newWebSourceUrl.value = '';
  settingsStore.config.web_sources = JSON.stringify(webSources.value);
};

const removeWebSource = (index: number) => {
  webSources.value.splice(index, 1);
  settingsStore.config.web_sources = JSON.stringify(webSources.value);
};

onMounted(async () => {
  await settingsStore.fetchConfig();
  initSourceConfig();

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

// Re-initialize source config when config is refetched (e.g., after import)
watch(() => settingsStore.config, (newConfig) => {
  if (newConfig) {
    initSourceConfig();
  }
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
  const safeQuota = Number.isFinite(quota) ? Math.max(1, Math.round(quota)) : 1;
  const existing = settingsStore.config.daily_quotas.find(q => q.language === lang);
  if (existing) {
    existing.quota = safeQuota;
  } else {
    settingsStore.config.daily_quotas.push({ language: lang, quota: safeQuota });
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

// Test reminder state
const testReminderSending = ref(false);
const testReminderResult = ref<{ success: boolean; message: string } | null>(null);

const handleTestReminder = async () => {
  testReminderSending.value = true;
  testReminderResult.value = null;
  try {
    const result = await sendTestReminder();
    testReminderResult.value = {
      success: true,
      message: `Test reminder sent via ${result.tier}`,
    };
  } catch (err: any) {
    testReminderResult.value = {
      success: false,
      message: err.response?.data?.detail || err.message || 'Failed to send test reminder',
    };
  } finally {
    testReminderSending.value = false;
    setTimeout(() => { testReminderResult.value = null; }, 5000);
  }
};
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

            <!-- Model Input (API) - no model name needed -->
          </div>
        </section>

        <!-- Exercise Sources Section -->
        <section id="exercise-sources" class="settings-section">
          <h2 class="section-heading">
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 6h16M4 12h16M4 18h16"/><circle cx="8" cy="6" r="1.5" fill="currentColor"/><circle cx="8" cy="12" r="1.5" fill="currentColor"/><circle cx="8" cy="18" r="1.5" fill="currentColor"/></svg>
            Exercise Sources
          </h2>
          <p class="section-description">
            Choose where exercises come from and in what order. Sources are used in priority order from top to bottom.
          </p>
          <div class="section-body">
            <div class="source-cards-container">
              <SourceCard
                v-for="(sourceKey, index) in sourcePriority"
                :key="sourceKey"
                :source-key="sourceKey"
                :label="sourceLabels[sourceKey]"
                :enabled="sourcesEnabled[sourceKey]"
                :disabled="enabledCount === 1 && sourcesEnabled[sourceKey]"
                disabled-tooltip="At least one exercise source must remain enabled."
                @update:enabled="onSourceToggle(sourceKey, $event)"
                @dragstart="onDragStart($event, index)"
                @dragover="onDragOver($event)"
                @drop="onDrop($event, index)"
              >
                <!-- GitHub Sub-Settings -->
                <template v-if="sourceKey === 'github'">
                  <h3 class="sub-heading">Custom Repositories</h3>

                  <div v-if="githubRepos.length > 0" class="source-list">
                    <div v-for="(repo, repoIdx) in githubRepos" :key="repoIdx" class="source-item">
                      <div class="source-item-fields">
                        <input
                          type="text"
                          v-model="repo.url"
                          placeholder="https://github.com/user/repo"
                          class="form-input"
                          @change="settingsStore.config.github_repos = JSON.stringify(githubRepos)"
                        />
                        <input
                          type="text"
                          v-model="repo.branch"
                          placeholder="main"
                          class="form-input w-40"
                          @change="settingsStore.config.github_repos = JSON.stringify(githubRepos)"
                        />
                      </div>
                      <button @click="removeGithubRepo(repoIdx)" class="remove-source-btn" title="Remove repository">
                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
                      </button>
                    </div>
                  </div>
                  <div v-else class="empty-state">
                    No custom repositories added. Add a repository URL above to include it as an exercise source.
                  </div>

                  <div class="add-source-form">
                    <input
                      type="text"
                      v-model="newGithubRepoUrl"
                      placeholder="https://github.com/user/repo"
                      :class="['form-input', urlValidationError['github-repo'] ? 'border-red-400' : '']"
                      @keyup.enter="addGithubRepo"
                    />
                    <input
                      type="text"
                      v-model="newGithubRepoBranch"
                      placeholder="main"
                      class="form-input w-40"
                      @keyup.enter="addGithubRepo"
                    />
                    <button @click="addGithubRepo" class="action-btn secondary">Add Repository</button>
                  </div>
                  <div v-if="urlValidationError['github-repo']" class="url-error">
                    {{ urlValidationError['github-repo'] }}
                  </div>
                </template>

                <!-- Web Sub-Settings -->
                <template v-else-if="sourceKey === 'web'">
                  <h3 class="sub-heading">Web Sources</h3>

                  <div v-if="webSources.length > 0" class="source-list">
                    <div v-for="(url, urlIdx) in webSources" :key="urlIdx" class="source-item source-item-single">
                      <span class="source-url-text">{{ url }}</span>
                      <button @click="removeWebSource(urlIdx)" class="remove-source-btn" title="Remove web source">
                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
                      </button>
                    </div>
                  </div>
                  <div v-else class="empty-state">
                    No web sources configured. Add a URL above or restore defaults to include web tutorials as exercise sources.
                  </div>

                  <div class="add-source-form">
                    <input
                      type="text"
                      v-model="newWebSourceUrl"
                      placeholder="https://example.com/tutorial"
                      :class="['form-input', urlValidationError['web-source'] ? 'border-red-400' : '']"
                      @keyup.enter="addWebSource"
                    />
                    <button @click="addWebSource" class="action-btn secondary">Add Web Source</button>
                  </div>
                  <div v-if="urlValidationError['web-source']" class="url-error">
                    {{ urlValidationError['web-source'] }}
                  </div>
                </template>
              </SourceCard>
            </div>
          </div>
        </section>

        <!-- Notifications Section -->
        <section id="notifications" class="settings-section">
          <h2 class="section-heading">
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
            Notifications
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

            <!-- Reminder Method -->
            <div class="form-group">
              <label class="form-label" for="reminder-method">Reminder Method</label>
              <select
                id="reminder-method"
                v-model="reminderMethod"
                class="form-select"
              >
                <option v-for="opt in reminderMethodOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
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

            <!-- Gmail-specific fields -->
            <template v-if="reminderMethod === 'gmail'">
              <hr class="section-divider" />
              <h3 class="sub-heading">Gmail OAuth Credentials</h3>
              <p class="hint-text">These are needed for Gmail API notification. The Connect button below handles the OAuth flow.</p>

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

              <div class="form-group">
                <GmailConnectPanel />
              </div>
            </template>

            <!-- SMTP-specific fields -->
            <template v-if="reminderMethod === 'smtp'">
              <hr class="section-divider" />
              <h3 class="sub-heading">SMTP Configuration</h3>

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
            </template>

            <!-- Test Reminder -->
            <hr class="section-divider" />
            <div class="form-group">
              <button
                @click="handleTestReminder"
                :disabled="testReminderSending || !settingsStore.config?.reminders_enabled"
                class="action-btn secondary"
              >
                <span v-if="testReminderSending" class="btn-spinner"></span>
                {{ testReminderSending ? 'Sending...' : 'Send Test Reminder' }}
              </button>
              <div v-if="testReminderResult" :class="['test-reminder-result', testReminderResult.success ? 'test-reminder-success' : 'test-reminder-error']">
                {{ testReminderResult.message }}
              </div>
              <p v-if="!settingsStore.config?.reminders_enabled" class="hint-text">
                Enable reminders first to send a test notification.
              </p>
            </div>
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

/* Test Reminder */
.test-reminder-result {
  margin-top: 0.5rem;
  font-size: 0.85rem;
  padding: 0.5rem 0.75rem;
  border-radius: 0.5rem;
}

.test-reminder-success {
  color: #16a34a;
  background-color: #f0fdf4;
  border: 1px solid #bbf7d0;
}

.test-reminder-error {
  color: #dc2626;
  background-color: #fef2f2;
  border: 1px solid #fecaca;
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

/* Source Cards Container */
.source-cards-container {
  display: flex;
  flex-direction: column;
}

.source-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  margin-bottom: 1rem;
}

.source-item {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  background-color: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  padding: 0.75rem;
}

.source-item-single {
  align-items: center;
}

.source-item-fields {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.source-url-text {
  flex: 1;
  font-size: 0.85rem;
  color: #374151;
  word-break: break-all;
  line-height: 1.4;
}

.remove-source-btn {
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
  flex-shrink: 0;
}

.remove-source-btn:hover {
  background-color: #fef2f2;
  color: #dc2626;
}

.add-source-form {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.25rem;
}

.url-error {
  color: #dc2626;
  font-size: 0.8rem;
  margin-top: 0.25rem;
}

.empty-state {
  font-size: 0.85rem;
  color: #9ca3af;
  padding: 0.5rem 0;
  margin-bottom: 0.5rem;
}

/* Drag visual feedback */
.source-card.opacity-50 {
  opacity: 0.5;
}

/* Utility widths */
.w-40 {
  width: 10rem;
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