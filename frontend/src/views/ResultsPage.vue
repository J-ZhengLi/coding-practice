<script setup lang="ts">
import { useRoute } from 'vue-router';

const route = useRoute();
const submissionId = route.params.id;
</script>

<template>
  <div class="results-page">
    <div class="results-container">
      <h1>Results</h1>
      <p>Submission ID: {{ submissionId }}</p>
      <p>Results page will be implemented in a future plan.</p>
    </div>
  </div>
</template>

<style scoped>
.results-page {
  min-height: 100vh;
  background-color: #f9fafb;
  padding: 2rem 1rem;
}

.results-container {
  max-width: 800px;
  margin: 0 auto;
  text-align: center;
  color: #6b7280;
}
</style>