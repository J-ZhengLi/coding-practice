<script setup lang="ts">
interface Props {
  label: string;
  sectionId: string;
  active: boolean;
  icon?: string;
}

interface Emits {
  (e: 'navigate', sectionId: string): void;
}

defineProps<Props>();
const emit = defineEmits<Emits>();

const handleClick = (sectionId: string) => {
  emit('navigate', sectionId);
};
</script>

<template>
  <button
    @click="handleClick(sectionId)"
    :class="[
      'w-full text-left px-4 py-2.5 text-sm transition-colors rounded-r-md',
      active
        ? 'border-l-4 border-blue-500 bg-blue-50 font-semibold text-blue-700'
        : 'border-l-4 border-transparent text-gray-600 hover:bg-gray-50 hover:text-gray-900'
    ]"
    :aria-current="active ? 'location' : undefined"
  >
    <span v-if="icon" class="mr-2" v-html="icon"></span>
    {{ label }}
  </button>
</template>