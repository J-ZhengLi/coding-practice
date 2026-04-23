<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useConfigStore, type UserConfig } from '../stores/config';
import LanguageSelection from './LanguageSelection.vue';
import SkillLevelSelection from './SkillLevelSelection.vue';
import DailyQuotaConfig from './DailyQuotaConfig.vue';
import AIModelSelection from './AIModelSelection.vue';
import ReviewAndConfirm from './ReviewAndConfirm.vue';

const router = useRouter();
const configStore = useConfigStore();

const currentStep = ref(1);
const totalSteps = 5;
const isSaving = ref(false);
const saveError = ref<string | null>(null);

// Form state - no defaults (per D-06)
const formData = ref({
  preferredLanguage: '',
  skillLevel: '',
  dailyQuotas: {
    python: 0,
    rust: 0,
    go: 0,
    cpp: 0,
  },
  aiModel: '',
  aiModelType: 'local' as 'local' | 'api',
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
    const config: UserConfig = {
      preferred_language: formData.value.preferredLanguage,
      skill_level: formData.value.skillLevel,
      daily_quotas: [
        { language: 'python', quota: formData.value.dailyQuotas.python },
        { language: 'rust', quota: formData.value.dailyQuotas.rust },
        { language: 'go', quota: formData.value.dailyQuotas.go },
        { language: 'cpp', quota: formData.value.dailyQuotas.cpp },
      ],
      ai_model: formData.value.aiModel,
      ai_model_type: formData.value.aiModelType,
    };

    await configStore.saveConfig(config);

    // Update store state
    configStore.isConfigured = true;
    configStore.config = config;

    // Redirect to dashboard
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
      v-model="formData.preferredLanguage"
      @next="nextStep"
    />

    <SkillLevelSelection
      v-if="currentStep === 2"
      v-model="formData.skillLevel"
      @next="nextStep"
      @back="prevStep"
    />

    <DailyQuotaConfig
      v-if="currentStep === 3"
      v-model="formData.dailyQuotas"
      @next="nextStep"
      @back="prevStep"
    />

    <AIModelSelection
      v-if="currentStep === 4"
      v-model="formData.aiModel"
      v-model:type="formData.aiModelType"
      @next="nextStep"
      @back="prevStep"
    />

    <ReviewAndConfirm
      v-if="currentStep === 5"
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