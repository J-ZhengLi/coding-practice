import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router';
import { useConfigStore } from '../stores/config';

// Placeholder views - will be implemented in Plan 04
const ConfigurationPage = () => import('../views/ConfigurationPage.vue').catch(() => {
  // Fallback if view doesn't exist yet
  return { template: '<div>Configuration Page (placeholder)</div>' };
});

const DashboardPage = () => import('../views/DashboardPage.vue').catch(() => {
  // Fallback if view doesn't exist yet
  return { template: '<div>Dashboard Page (placeholder)</div>' };
});

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'root',
    redirect: '/dashboard',
  },
  {
    path: '/configuration',
    name: 'configuration',
    component: ConfigurationPage,
    meta: { title: 'Configuration' },
  },
  {
    path: '/dashboard',
    name: 'dashboard',
    component: DashboardPage,
    meta: { requiresConfig: true, title: 'Dashboard' },
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

// Navigation guard for configuration requirement
router.beforeEach(async (to, from, next) => {
  const configStore = useConfigStore();

  // Check if configuration status is known
  if (!configStore.loading && configStore.isConfigured === false && to.meta.requiresConfig) {
    // Not configured, redirect to configuration page
    next({ name: 'configuration' });
  } else if (configStore.isConfigured === true && to.name === 'configuration') {
    // Already configured, redirect to dashboard
    next({ name: 'dashboard' });
  } else {
    next();
  }
});

export default router;