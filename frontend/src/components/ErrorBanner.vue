<script setup lang="ts">
defineProps<{
  message: string;
}>();

const emit = defineEmits<{
  retry: [];
  dismiss: [];
}>();
</script>

<template>
  <div class="error-banner" role="alert">
    <div class="error-content">
      <h4 class="error-heading">Evaluation failed</h4>
      <p class="error-message">{{ message }}</p>
    </div>
    <div class="error-actions">
      <button class="retry-btn" @click="emit('retry')">Retry Submission</button>
      <button class="dismiss-btn" @click="emit('dismiss')" aria-label="Dismiss error">&times;</button>
    </div>
  </div>
</template>

<style scoped>
.error-banner {
  background-color: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 0.75rem;
  padding: 1rem 1.5rem;
  margin-bottom: 1rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.error-content {
  flex: 1;
}
.error-heading {
  font-size: 0.95rem;
  font-weight: 600;
  color: #991b1b;
  margin: 0 0 0.25rem 0;
}
.error-message {
  color: #991b1b;
  font-size: 0.85rem;
  margin: 0;
}
.error-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.retry-btn {
  background-color: #ef4444;
  color: white;
  border: none;
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
}
.retry-btn:hover { background-color: #dc2626; }
.dismiss-btn {
  background: none;
  border: none;
  font-size: 1.25rem;
  color: #991b1b;
  cursor: pointer;
  padding: 0 0.25rem;
}
</style>