<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useConfigStore, type UserConfig, type LanguageSkillLevel, type LanguageQuota } from '../stores/config';
import LanguageSelection from './LanguageSelection.vue';
import LanguageConfigStep from './LanguageConfigStep.vue';
import AIModelSelection from './AIModelSelection.vue';
import ReviewAndConfirm from './ReviewAndConfirm.vue';

const router = useRouter();
const configStore = useConfigStore();

const currentStep = ref(1);
const totalSteps = 4;
const isSaving = ref(false);
const saveError = ref<string | null>(null);

interface LanguageConfig {
  language: string;
  skill_level: string;
  quota: number;
}

const formData = ref({
  selectedLanguages: [] as string[],
  languageConfigs: [] as LanguageConfig[],
  aiModel: '',
  aiModelType: '' as string,
});

const nextStep = () => {
  if (currentStep.value < totalSteps) {
    currentStep.value++;
  }
};

const prevStep = () => {
  if (currentStep.value > 1) {
    currentStep.value--;
  }
};

const finishWizard = async () => {
  isSaving.value = true;
  saveError.value = null;

  try {
    const skillLevels: LanguageSkillLevel[] = formData.value.languageConfigs.map(c => ({
      language: c.language,
      skill_level: c.skill_level,
    }));

    const dailyQuotas: LanguageQuota[] = formData.value.languageConfigs.map(c => ({
      language: c.language,
      quota: c.quota,
    }));

    // Map display model types to backend API types: ollama_local/ollama_cloud → local, api stays api
    const apiModelType = formData.value.aiModelType.startsWith('ollama_') ? 'local' : formData.value.aiModelType;

    const config: UserConfig = {
      preferred_language: formData.value.selectedLanguages[0] || '',
      skill_levels: skillLevels,
      daily_quotas: dailyQuotas,
      ai_model: formData.value.aiModel,
      ai_model_type: apiModelType,
    };

    await configStore.saveConfig(config);

    configStore.isConfigured = true;
    configStore.config = config;

    router.push({ name: 'dashboard' });
  } catch (error: any) {
    saveError.value = error.response?.data?.detail || error.message || 'Failed to save configuration';
    console.error('Failed to save configuration:', error);
  } finally {
    isSaving.value = false;
  }
};
</script>

<template>
  <div class="config-wizard">
    <div class="wizard-header">
      <div class="progress-indicator">
        Step {{ currentStep }} of {{ totalSteps }}
      </div>
      <div class="progress-bar">
        <div
          class="progress-fill"
          :style="{ width: `${(currentStep / totalSteps) * 100}%` }"
        ></div>
      </div>
    </div>

    <div v-if="saveError" class="error-alert">
      {{ saveError }}
    </div>

    <LanguageSelection
      v-if="currentStep === 1"
      v-model="formData.selectedLanguages"
      @next="nextStep"
    />

    <LanguageConfigStep
      v-if="currentStep === 2"
      :selected-languages="formData.selectedLanguages"
      v-model="formData.languageConfigs"
      @next="nextStep"
      @back="prevStep"
    />

    <AIModelSelection
      v-if="currentStep === 3"
      v-model="formData.aiModel"
      v-model:type="formData.aiModelType"
      @next="nextStep"
      @back="prevStep"
    />

    <ReviewAndConfirm
      v-if="currentStep === 4"
      :config="formData"
      @finish="finishWizard"
      @back="prevStep"
    />

    <div v-if="isSaving" class="saving-overlay">
      <div class="saving-spinner"></div>
      <p>Saving configuration...</p>
    </div>
  </div>
</template>

<style scoped>
.config-wizard {
  max-width: 1000px;
  margin: 0 auto;
  padding: 2rem;
  position: relative;
}

.wizard-header {
  margin-bottom: 2rem;
}

.progress-indicator {
  text-align: center;
  font-size: 1.1rem;
  font-weight: 600;
  color: #6b7280;
  margin-bottom: 0.75rem;
}

.progress-bar {
  width: 100%;
  height: 0.5rem;
  background-color: #e5e7eb;
  border-radius: 9999px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background-color: #3b82f6;
  transition: width 0.3s ease;
}

.error-alert {
  background-color: #fef2f2;
  border: 1px solid #fecaca;
  color: #991b1b;
  padding: 1rem;
  border-radius: 0.5rem;
  margin-bottom: 1.5rem;
}

.saving-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(255, 255, 255, 0.9);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.saving-spinner {
  width: 3rem;
  height: 3rem;
  border: 3px solid #e5e7eb;
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 1rem;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.saving-overlay p {
  font-size: 1.1rem;
  color: #6b7280;
}
</style>