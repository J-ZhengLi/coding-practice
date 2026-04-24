<script setup lang="ts">
defineProps<{
  strengths: string[];
  improvements: string[];
}>();
</script>

<template>
  <div class="feedback-container">
    <div class="feedback-section strengths">
      <h4 class="feedback-heading">What you did well</h4>
      <ul class="feedback-list">
        <li v-for="(item, i) in strengths" :key="i">{{ item }}</li>
      </ul>
      <p v-if="strengths.length === 0" class="feedback-empty">No specific strengths noted.</p>
    </div>
    <div class="feedback-section improvements">
      <h4 class="feedback-heading">Areas for improvement</h4>
      <ul class="feedback-list">
        <li v-for="(item, i) in improvements" :key="i">{{ item }}</li>
      </ul>
      <p v-if="improvements.length === 0" class="feedback-empty">No specific improvements noted.</p>
    </div>
  </div>
</template>

<style scoped>
.feedback-container {
  margin-top: 1rem;
}

.feedback-section {
  padding: 16px;
  border-radius: 0.75rem;
  margin-bottom: 12px;
}

.strengths {
  border-left: 4px solid #22c55e;
  background: #f0fdf4;
}

.improvements {
  border-left: 4px solid #d97706;
  background: #fffbeb;
}

.feedback-heading {
  font-size: 1rem;
  font-weight: 600;
  margin: 0 0 0.5rem 0;
}

.strengths .feedback-heading {
  color: #166534;
}

.improvements .feedback-heading {
  color: #854d0e;
}

.feedback-list {
  list-style: disc;
  padding-left: 20px;
  margin: 0;
}

.feedback-list li {
  font-size: 0.9rem;
  line-height: 1.6;
  color: #374151;
}

.feedback-empty {
  color: #9ca3af;
  font-style: italic;
  margin: 0;
  font-size: 0.9rem;
}
</style>