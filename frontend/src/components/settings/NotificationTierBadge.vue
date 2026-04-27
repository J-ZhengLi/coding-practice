<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  tier: 'gmail' | 'smtp' | 'desktop' | 'none';
}

const props = defineProps<Props>();

const tierInfo = computed(() => {
  const map: Record<string, { label: string; icon: string; color: string }> = {
    gmail: {
      label: 'Gmail API',
      icon: '<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 4L12 13 2 4"/></svg>',
      color: 'bg-red-50 text-red-700 border-red-200',
    },
    smtp: {
      label: 'SMTP',
      icon: '<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 4L12 13 2 4"/></svg>',
      color: 'bg-blue-50 text-blue-700 border-blue-200',
    },
    desktop: {
      label: 'Desktop notifications',
      icon: '<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/></svg>',
      color: 'bg-green-50 text-green-700 border-green-200',
    },
    none: {
      label: 'None configured',
      icon: '<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M15 9l-6 6M9 9l6 6"/></svg>',
      color: 'bg-gray-50 text-gray-500 border-gray-200',
    },
  };
  return map[props.tier] || map.none;
});
</script>

<template>
  <div
    :class="['inline-flex items-center gap-2 px-3 py-1.5 rounded-lg border text-sm', tierInfo.color]"
  >
    <span v-html="tierInfo.icon"></span>
    <span class="font-medium">Currently using: {{ tierInfo.label }}</span>
  </div>
</template>