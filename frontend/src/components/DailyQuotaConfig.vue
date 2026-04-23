<script setup lang="ts">
import { ref, computed } from 'vue';

interface Props {
  modelValue: {
    python: number;
    rust: number;
    go: number;
    cpp: number;
  };
}

interface Emits {
  (e: 'update:modelValue', value: Props['modelValue']): void;
  (e: 'next'): void;
  (e: 'back'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const quotas = ref({
  python: props.modelValue.python,
  rust: props.modelValue.rust,
  go: props.modelValue.go,
  cpp: props.modelValue.cpp,
});

const updateQuota = (language: string, value: number) => {
  quotas.value[language as keyof typeof quotas.value] = value;
  emit('update:modelValue', { ...quotas.value });
};

const canProceed = computed(() => {
  return Object.values(quotas.value).every(q => q > 0);
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
  <div class="daily-quota-config">
    <h2 class="step-title">Set Daily Exercise Quotas</h2>
    <p class="step-description">How many exercises do you want to complete each day for each language? Minimum 1 per language.</p>

    <div class="quota-list">
      <div v-for="(quota, lang) in quotas" :key="lang" class="quota-item">
        <label :for="`quota-${lang}`" class="quota-label">
          {{ lang.charAt(0).toUpperCase() + lang.slice(1) }}
        </label>
        <input
          :id="`quota-${lang}`"
          v-model.number="quotas[lang]"
          type="number"
          min="1"
          max="20"
          class="quota-input"
          :class="{ invalid: quotas[lang] <= 0 }"
        />
        <span class="quota-suffix">exercises/day</span>
      </div>
    </div>

    <div class="button-group">
      <button class="btn btn-secondary" @click="goBack">Back</button>
      <button class="btn btn-primary" :disabled="!canProceed" @click="nextStep">
        Next
      </button>
    </div>

    <p v-if="!canProceed" class="error-message">
      All quotas must be at least 1 exercise per day.
    </p>
  </div>
</template>

<style scoped>
.daily-quota-config {
  max-width: 600px;
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

.quota-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-bottom: 2rem;
}

.quota-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem;
  border: 2px solid #e5e7eb;
  border-radius: 0.5rem;
}

.quota-label {
  font-weight: 600;
  color: #1f2937;
  min-width: 60px;
}

.quota-input {
  flex: 1;
  padding: 0.5rem;
  border: 2px solid #d1d5db;
  border-radius: 0.375rem;
  font-size: 1rem;
  max-width: 100px;
}

.quota-input.invalid {
  border-color: #ef4444;
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

.error-message {
  color: #ef4444;
  font-size: 0.9rem;
  margin-top: 1rem;
}
</style>