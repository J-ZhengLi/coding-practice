<script setup lang="ts">
interface Props {
  modelValue: string;
}

interface Emits {
  (e: 'update:modelValue', value: string): void;
  (e: 'next'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const languages = [
  { value: 'python', label: 'Python', description: 'Great for beginners, widely used in data science and web development' },
  { value: 'rust', label: 'Rust', description: 'Systems programming with memory safety guarantees' },
  { value: 'go', label: 'Go', description: 'Simple and efficient for building scalable services' },
  { value: 'cpp', label: 'C++', description: 'High-performance language for systems and game development' },
];

const selectLanguage = (language: string) => {
  emit('update:modelValue', language);
  emit('next');
};
</script>

<template>
  <div class="language-selection">
    <h2 class="step-title">Choose Your Preferred Programming Language</h2>
    <p class="step-description">Select the language you want to focus on for your daily practice.</p>

    <div class="language-grid">
      <div
        v-for="lang in languages"
        :key="lang.value"
        class="language-card"
        :class="{ selected: modelValue === lang.value }"
        @click="selectLanguage(lang.value)"
      >
        <h3>{{ lang.label }}</h3>
        <p>{{ lang.description }}</p>
      </div>
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
}

.language-card {
  padding: 1.5rem;
  border: 2px solid #e5e7eb;
  border-radius: 0.75rem;
  cursor: pointer;
  transition: all 0.2s;
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
</style>