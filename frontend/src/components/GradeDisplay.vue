<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  letterGrade: string;
  score: number;
  isPartial: boolean;
}>();

const gradeStyles: Record<string, { bg: string; text: string; border: string }> = {
  A: { bg: '#f0fdf4', text: '#166534', border: '#bbf7d0' },
  B: { bg: '#f7fee7', text: '#3f6212', border: '#d9f99d' },
  C: { bg: '#fefce8', text: '#854d0e', border: '#fef08a' },
  D: { bg: '#fff7ed', text: '#9a3412', border: '#fed7aa' },
  F: { bg: '#fef2f2', text: '#991b1b', border: '#fecaca' },
};

const currentStyle = computed(() => {
  return gradeStyles[props.letterGrade] || gradeStyles['F'];
});

const containerStyle = computed(() => ({
  backgroundColor: currentStyle.value.bg,
  color: currentStyle.value.text,
  borderColor: currentStyle.value.border,
}));
</script>

<template>
  <div
    class="grade-display"
    :style="containerStyle"
    :aria-label="`Grade: ${letterGrade}, Structural Completeness: ${score} out of 100`"
  >
    <div class="letter-grade">{{ letterGrade }}</div>
    <div class="score-section">
      <span class="score-value">{{ score }}</span>
      <span class="score-label">Structural Completeness</span>
    </div>
    <div v-if="isPartial" class="partial-badge">Partial implementation</div>
  </div>
</template>

<style scoped>
.grade-display {
  text-align: center;
  padding: 2rem;
  border-radius: 1rem;
  border: 2px solid;
  margin-bottom: 1.5rem;
}

.letter-grade {
  font-size: 4.5rem;
  font-weight: 600;
  line-height: 1.2;
  margin-bottom: 0.5rem;
}

.score-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
}

.score-value {
  font-size: 1.25rem;
  font-weight: 600;
}

.score-label {
  font-size: 0.875rem;
  color: #6b7280;
}

.partial-badge {
  display: inline-block;
  margin-top: 0.75rem;
  padding: 4px 12px;
  border-radius: 9999px;
  font-size: 0.75rem;
  font-weight: 600;
  color: #f59e0b;
  background-color: #fef3c7;
}
</style>