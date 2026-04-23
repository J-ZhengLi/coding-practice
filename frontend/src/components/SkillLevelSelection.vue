<script setup lang="ts">
interface Props {
  modelValue: string;
}

interface Emits {
  (e: 'update:modelValue', value: string): void;
  (e: 'next'): void;
  (e: 'back'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const skillLevels = [
  {
    value: 'beginner',
    label: 'Beginner',
    description: 'New to programming or this language. Exercises focus on basic syntax and simple problems.',
  },
  {
    value: 'intermediate',
    label: 'Intermediate',
    description: 'Comfortable with basics. Exercises involve algorithms, data structures, and moderate complexity.',
  },
  {
    value: 'advanced',
    label: 'Advanced',
    description: 'Experienced developer. Exercises cover complex algorithms, system design, and optimization.',
  },
];

const selectLevel = (level: string) => {
  emit('update:modelValue', level);
  emit('next');
};

const goBack = () => {
  emit('back');
};
</script>

<template>
  <div class="skill-level-selection">
    <h2 class="step-title">Select Your Skill Level</h2>
    <p class="step-description">This helps us tailor exercises to your experience.</p>

    <div class="skill-level-grid">
      <div
        v-for="level in skillLevels"
        :key="level.value"
        class="skill-level-card"
        :class="{ selected: modelValue === level.value }"
        @click="selectLevel(level.value)"
      >
        <h3>{{ level.label }}</h3>
        <p>{{ level.description }}</p>
      </div>
    </div>

    <button class="btn btn-secondary" @click="goBack">Back</button>
  </div>
</template>

<style scoped>
.skill-level-selection {
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

.skill-level-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.skill-level-card {
  padding: 1.5rem;
  border: 2px solid #e5e7eb;
  border-radius: 0.75rem;
  cursor: pointer;
  transition: all 0.2s;
}

.skill-level-card:hover {
  border-color: #3b82f6;
  background-color: #f0f9ff;
}

.skill-level-card.selected {
  border-color: #3b82f6;
  background-color: #eff6ff;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
}

.skill-level-card h3 {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #1f2937;
}

.skill-level-card p {
  color: #6b7280;
  font-size: 0.95rem;
  line-height: 1.5;
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
</style>