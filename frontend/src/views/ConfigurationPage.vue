<script setup lang="ts">
import { onMounted } from 'vue';
import ConfigWizard from '../components/ConfigWizard.vue';
import { useConfigStore } from '../stores/config';
import { useRouter } from 'vue-router';

const configStore = useConfigStore();
const router = useRouter();

onMounted(async () => {
  // Check if already configured
  await configStore.checkConfigured();
  if (configStore.isConfigured) {
    // Redirect to dashboard if already configured
    router.push({ name: 'dashboard' });
  }
});
</script>

<template>
  <div class="configuration-page">
    <div class="page-header">
      <h1>Welcome to AI Programming Learning Assistant</h1>
      <p>Let's set up your personalized learning experience.</p>
    </div>
    <ConfigWizard />
  </div>
</template>

<style scoped>
.configuration-page {
  min-height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 2rem 1rem;
}

.page-header {
  text-align: center;
  color: white;
  margin-bottom: 3rem;
}

.page-header h1 {
  font-size: 2.5rem;
  font-weight: 700;
  margin-bottom: 0.5rem;
}

.page-header p {
  font-size: 1.25rem;
  opacity: 0.9;
}

@media (max-width: 768px) {
  .page-header h1 {
    font-size: 1.75rem;
  }

  .page-header p {
    font-size: 1rem;
  }
}
</style>