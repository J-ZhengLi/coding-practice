<script setup lang="ts">
import { onMounted } from 'vue';
import { useConfigStore } from '../stores/config';
import { useRouter } from 'vue-router';

const configStore = useConfigStore();
const router = useRouter();

onMounted(async () => {
  // Ensure configuration is loaded
  if (!configStore.config) {
    await configStore.fetchConfig();
  }

  // Redirect to configuration if not configured
  if (!configStore.isConfigured) {
    router.push({ name: 'configuration' });
  }
});

const skillLevelLabel = (level: string): string => {
  const labels: Record<string, string> = {
    beginner: 'Beginner',
    intermediate: 'Intermediate',
    advanced: 'Advanced',
  };
  return labels[level] || level;
};

const languageLabel = (lang: string): string => {
  const labels: Record<string, string> = {
    python: 'Python',
    rust: 'Rust',
    go: 'Go',
    cpp: 'C++',
  };
  return labels[lang] || lang;
};
</script>

<template>
  <div class="dashboard-page">
    <div class="dashboard-container">
      <div class="dashboard-header">
        <h1>Dashboard</h1>
        <p>Your personalized learning space</p>
      </div>

      <div v-if="configStore.config" class="config-summary">
        <h2>Configuration Complete</h2>
        <p>You're all set to start your programming journey!</p>

        <div class="summary-grid">
          <div class="summary-card">
            <h3>Preferred Language</h3>
            <p class="summary-value">{{ languageLabel(configStore.config.preferred_language) }}</p>
          </div>

          <div class="summary-card">
            <h3>Skill Level</h3>
            <p class="summary-value">{{ skillLevelLabel(configStore.config.skill_level) }}</p>
          </div>

          <div class="summary-card">
            <h3>AI Model</h3>
            <p class="summary-value">{{ configStore.config.ai_model }}</p>
          </div>
        </div>

        <div class="quota-summary">
          <h3>Daily Exercise Quotas</h3>
          <ul>
            <li v-for="quota in configStore.config.daily_quotas" :key="quota.language">
              <strong>{{ languageLabel(quota.language) }}:</strong> {{ quota.quota }} exercises/day
            </li>
          </ul>
        </div>

        <div class="next-steps">
          <h3>What's Next?</h3>
          <p>Phase 2 will add material acquisition and exercise generation. Stay tuned!</p>
        </div>
      </div>

      <div v-else class="loading">
        Loading your configuration...
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard-page {
  min-height: 100vh;
  background-color: #f9fafb;
  padding: 2rem 1rem;
}

.dashboard-container {
  max-width: 1000px;
  margin: 0 auto;
}

.dashboard-header {
  text-align: center;
  margin-bottom: 3rem;
}

.dashboard-header h1 {
  font-size: 2.5rem;
  font-weight: 700;
  color: #1f2937;
  margin-bottom: 0.5rem;
}

.dashboard-header p {
  font-size: 1.25rem;
  color: #6b7280;
}

.config-summary {
  background-color: white;
  border-radius: 1rem;
  padding: 2rem;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.config-summary h2 {
  font-size: 1.75rem;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 0.5rem;
}

.config-summary > p {
  font-size: 1.1rem;
  color: #6b7280;
  margin-bottom: 2rem;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.summary-card {
  background-color: #f9fafb;
  border-radius: 0.75rem;
  padding: 1.5rem;
  text-align: center;
}

.summary-card h3 {
  font-size: 0.875rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.5rem;
}

.summary-value {
  font-size: 1.25rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0;
}

.quota-summary {
  background-color: #f9fafb;
  border-radius: 0.75rem;
  padding: 1.5rem;
  margin-bottom: 2rem;
}

.quota-summary h3 {
  font-size: 1rem;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 1rem;
}

.quota-summary ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

.quota-summary li {
  padding: 0.5rem 0;
  color: #374151;
  font-size: 1rem;
}

.next-steps {
  background-color: #eff6ff;
  border-radius: 0.75rem;
  padding: 1.5rem;
  border-left: 4px solid #3b82f6;
}

.next-steps h3 {
  font-size: 1rem;
  font-weight: 600;
  color: #1e40af;
  margin-bottom: 0.5rem;
}

.next-steps p {
  color: #1e3a8a;
  margin: 0;
}

.loading {
  text-align: center;
  padding: 3rem;
  color: #6b7280;
  font-size: 1.1rem;
}

@media (max-width: 768px) {
  .dashboard-header h1 {
    font-size: 1.75rem;
  }

  .dashboard-header p {
    font-size: 1rem;
  }

  .summary-grid {
    grid-template-columns: 1fr;
  }
}
</style>