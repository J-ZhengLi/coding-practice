<script setup lang="ts">
import { computed } from 'vue';
import ScoreTrend from './ScoreTrend.vue';

const props = defineProps<{
  dailyAverage: number | null;
  letterGrade: string | null;
  completionCount: number;
  scoreTrend: { date: string; avg_score: number }[];
}>();

const gradeBadgeClass = computed(() => {
  const map: Record<string, string> = {
    A: 'grade-a-badge',
    B: 'grade-b-badge',
    C: 'grade-c-badge',
    D: 'grade-d-badge',
    F: 'grade-f-badge',
  };
  return map[props.letterGrade || ''] || '';
});
</script>

<template>
  <div class="progress-metrics">
    <!-- Daily Average Card -->
    <div class="metric-card">
      <h4 class="metric-label">Daily Average</h4>
      <p class="metric-value" v-if="dailyAverage !== null">{{ Math.round(dailyAverage) }}</p>
      <span v-if="letterGrade" :class="['grade-badge', gradeBadgeClass]">{{ letterGrade }}</span>
      <p v-if="dailyAverage === null" class="metric-empty">No data</p>
    </div>
    <!-- Completion Count Card -->
    <div class="metric-card">
      <h4 class="metric-label">Exercises Completed</h4>
      <p class="metric-value">{{ completionCount }}</p>
    </div>
    <!-- 7-Day Trend Card -->
    <div class="metric-card">
      <h4 class="metric-label">7-Day Trend</h4>
      <ScoreTrend v-if="scoreTrend.length > 0" :scores="scoreTrend" />
      <p v-else class="metric-empty">No data</p>
    </div>
  </div>
</template>

<style scoped>
.progress-metrics {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
}

.metric-card {
  background: #f9fafb;
  border-radius: 0.75rem;
  padding: 1.5rem;
  text-align: center;
}

.metric-label {
  font-size: 0.875rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  margin-bottom: 0.5rem;
}

.metric-value {
  font-size: 2rem;
  font-weight: 700;
  color: #3b82f6;
  margin: 0;
}

.grade-badge {
  display: inline-block;
  padding: 2px 12px;
  border-radius: 9999px;
  font-size: 0.75rem;
  font-weight: 600;
}

.grade-a-badge {
  background-color: #f0fdf4;
  color: #166534;
}

.grade-b-badge {
  background-color: #f7fee7;
  color: #3f6212;
}

.grade-c-badge {
  background-color: #fefce8;
  color: #854d0e;
}

.grade-d-badge {
  background-color: #fff7ed;
  color: #9a3412;
}

.grade-f-badge {
  background-color: #fef2f2;
  color: #991b1b;
}

.metric-empty {
  color: #9ca3af;
  font-size: 0.875rem;
  font-style: italic;
}

@media (max-width: 768px) {
  .progress-metrics {
    grid-template-columns: 1fr;
  }
}
</style>