import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router';
import { useConfigStore } from '../stores/config';

const ConfigurationPage = () => import('../views/ConfigurationPage.vue').catch(() => {
  return { template: '<div>Configuration Page (placeholder)</div>' };
});

const DashboardPage = () => import('../views/DashboardPage.vue').catch(() => {
  return { template: '<div>Dashboard Page (placeholder)</div>' };
});

// Lazy-loaded per Pitfall 4: Monaco Editor adds ~2-4MB, must be code-split
const ExerciseEditorPage = () => import('../views/ExerciseEditorPage.vue');
const ResultsPage = () => import('../views/ResultsPage.vue');
const SettingsPage = () => import('../views/SettingsPage.vue').catch(() => {
  return { template: '<div>Settings Page (placeholder)</div>' };
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
  {
    path: '/exercise/:id',
    name: 'exercise-editor',
    component: ExerciseEditorPage,
    meta: { requiresConfig: true, title: 'Exercise Editor' },
  },
  {
    path: '/results/:id',
    name: 'results',
    component: ResultsPage,
    meta: { requiresConfig: true, title: 'Results' },
  },
  {
    path: '/settings',
    name: 'settings',
    component: SettingsPage,
    meta: { requiresConfig: true, title: 'Settings' },
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