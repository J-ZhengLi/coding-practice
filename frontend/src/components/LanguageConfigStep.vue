<script setup lang="ts">
import { ref, computed, watch } from 'vue';

interface LanguageConfig {
  language: string;
  skill_level: string;
  quota: number;
}

interface Props {
  selectedLanguages: string[];
  modelValue: LanguageConfig[];
}

interface Emits {
  (e: 'update:modelValue', value: LanguageConfig[]): void;
  (e: 'next'): void;
  (e: 'back'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const skillLevels = [
  { value: 'beginner', label: 'Beginner', description: 'Basic syntax and simple problems' },
  { value: 'intermediate', label: 'Intermediate', description: 'Algorithms and data structures' },
  { value: 'advanced', label: 'Advanced', description: 'Complex algorithms and optimization' },
];

const languageLabels: Record<string, string> = {
  python: 'Python',
  rust: 'Rust',
  go: 'Go',
  cpp: 'C++',
};

const configs = ref<LanguageConfig[]>(
  props.selectedLanguages.map(lang => {
    const existing = props.modelValue.find(c => c.language === lang);
    return existing || { language: lang, skill_level: '', quota: 1 };
  })
);

watch(() => props.selectedLanguages, (newLangs) => {
  configs.value = newLangs.map(lang => {
    const existing = configs.value.find(c => c.language === lang);
    return existing || { language: lang, skill_level: '', quota: 1 };
  });
  emitUpdate();
});

const emitUpdate = () => {
  emit('update:modelValue', [...configs.value]);
};

const setSkillLevel = (language: string, level: string) => {
  const config = configs.value.find(c => c.language === language);
  if (config) {
    config.skill_level = level;
    emitUpdate();
  }
};

const setQuota = (language: string, quota: number) => {
  const config = configs.value.find(c => c.language === language);
  if (config) {
    config.quota = Math.max(1, quota);
    emitUpdate();
  }
};

const canProceed = computed(() => {
  return configs.value.length > 0 && configs.value.every(c => c.skill_level !== '');
});

const nextStep = () => {
  if (canProceed.value) {
    emit('next');
  }
};

const goBack = () => {
  emit('back');
};
</script>

<template>
  <div class="language-config-step">
    <h2 class="step-title">Configure Your Languages</h2>
    <p class="step-description">Set your skill level and daily exercise quota for each selected language.</p>

    <div class="language-config-list">
      <div v-for="config in configs" :key="config.language" class="language-config-card">
        <h3 class="language-name">{{ languageLabels[config.language] || config.language }}</h3>

        <div class="config-section">
          <label class="config-label">Skill Level</label>
          <div class="skill-level-options">
            <button
              v-for="level in skillLevels"
              :key="level.value"
              class="skill-btn"
              :class="{ selected: config.skill_level === level.value }"
              @click="setSkillLevel(config.language, level.value)"
              :title="level.description"
            >
              {{ level.label }}
            </button>
          </div>
        </div>

        <div class="config-section">
          <label class="config-label" :for="`quota-${config.language}`">Daily Quota</label>
          <div class="quota-control">
            <input
              :id="`quota-${config.language}`"
              type="number"
              min="1"
              max="20"
              :value="config.quota"
              @input="setQuota(config.language, ($event.target as HTMLInputElement).valueAsNumber)"
              class="quota-input"
            />
            <span class="quota-suffix">exercises/day</span>
          </div>
        </div>
      </div>
    </div>

    <div class="button-group">
      <button class="btn btn-secondary" @click="goBack">Back</button>
      <button class="btn btn-primary" :disabled="!canProceed" @click="nextStep">
        Next
      </button>
    </div>
  </div>
</template>

<style scoped>
.language-config-step {
  max-width: 700px;
  margin: 0 auto;
}

.step-title {
  font-size: 1.75rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #1f2937;
}

.step-description {
  color: #6b7280;
  margin-bottom: 2rem;
  font-size: 1.1rem;
}

.language-config-list {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.language-config-card {
  padding: 1.5rem;
  border: 2px solid #e5e7eb;
  border-radius: 0.75rem;
}

.language-name {
  font-size: 1.25rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0 0 1rem 0;
}

.config-section {
  margin-bottom: 1rem;
}

.config-section:last-child {
  margin-bottom: 0;
}

.config-label {
  display: block;
  font-size: 0.875rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.5rem;
}

.skill-level-options {
  display: flex;
  gap: 0.75rem;
}

.skill-btn {
  flex: 1;
  padding: 0.5rem 1rem;
  border: 2px solid #e5e7eb;
  border-radius: 0.5rem;
  background: white;
  cursor: pointer;
  font-size: 0.9rem;
  font-weight: 600;
  color: #374151;
  transition: all 0.2s;
}

.skill-btn:hover {
  border-color: #3b82f6;
  background-color: #f0f9ff;
}

.skill-btn.selected {
  border-color: #3b82f6;
  background-color: #eff6ff;
  color: #1d4ed8;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
}

.quota-control {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.quota-input {
  width: 80px;
  padding: 0.5rem;
  border: 2px solid #d1d5db;
  border-radius: 0.375rem;
  font-size: 1rem;
}

.quota-input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
}

.quota-suffix {
  color: #6b7280;
  font-size: 0.9rem;
}

.button-group {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
}

.btn {
  padding: 0.75rem 1.5rem;
  border-radius: 0.5rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
  font-size: 1rem;
}

.btn-secondary {
  background-color: #e5e7eb;
  color: #1f2937;
}

.btn-secondary:hover {
  background-color: #d1d5db;
}

.btn-primary {
  background-color: #3b82f6;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #2563eb;
}

.btn-primary:disabled {
  background-color: #9ca3af;
  cursor: not-allowed;
}
</style>