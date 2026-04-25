<script setup lang="ts">
interface Props {
  modelValue: string[];
}

interface Emits {
  (e: 'update:modelValue', value: string[]): void;
  (e: 'next'): void;
  (e: 'back'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const languages = [
  { value: 'python', label: 'Python', description: 'Great for beginners, widely used in data science and web development' },
  { value: 'rust', label: 'Rust', description: 'Systems programming with memory safety guarantees' },
  { value: 'go', label: 'Go', description: 'Simple and efficient for building scalable services' },
  { value: 'cpp', label: 'C++', description: 'High-performance language for systems and game development' },
];

const toggleLanguage = (language: string) => {
  const current = [...props.modelValue];
  const index = current.indexOf(language);
  if (index >= 0) {
    current.splice(index, 1);
  } else {
    current.push(language);
  }
  emit('update:modelValue', current);
};

const canProceed = computed(() => props.modelValue.length > 0);

const nextStep = () => {
  if (canProceed.value) {
    emit('next');
  }
};

const goBack = () => {
  emit('back');
};

import { computed } from 'vue';
</script>

<template>
  <div class="language-selection">
    <h2 class="step-title">Choose The Programming Language You Want To Learn</h2>
    <p class="step-description">Select one or more languages. You can configure skill level and daily quota for each.</p>

    <div class="language-grid">
      <div
        v-for="lang in languages"
        :key="lang.value"
        class="language-card"
        :class="{ selected: modelValue.includes(lang.value) }"
        @click="toggleLanguage(lang.value)"
      >
        <div class="card-check">
          <span v-if="modelValue.includes(lang.value)" class="check-mark">&#10003;</span>
        </div>
        <h3>{{ lang.label }}</h3>
        <p>{{ lang.description }}</p>
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
.language-selection {
  max-width: 800px;
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

.language-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.language-card {
  padding: 1.5rem;
  border: 2px solid #e5e7eb;
  border-radius: 0.75rem;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.language-card:hover {
  border-color: #3b82f6;
  background-color: #f0f9ff;
}

.language-card.selected {
  border-color: #3b82f6;
  background-color: #eff6ff;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
}

.card-check {
  position: absolute;
  top: 1rem;
  right: 1rem;
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.check-mark {
  color: #3b82f6;
  font-size: 1.25rem;
  font-weight: 700;
}

.language-card h3 {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #1f2937;
}

.language-card p {
  color: #6b7280;
  font-size: 0.95rem;
  line-height: 1.5;
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