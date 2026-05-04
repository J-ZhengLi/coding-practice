<script setup lang="ts">
interface Props {
  modelValue: boolean;
  label?: string;
  id?: string;
  disabled?: boolean;
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const toggle = () => {
  emit('update:modelValue', !props.modelValue);
};
</script>

<template>
  <label
    class="relative inline-flex items-center"
    :class="{ 'cursor-pointer': !disabled, 'cursor-not-allowed': disabled }"
    :for="id"
  >
    <input
      :id="id"
      type="checkbox"
      :checked="modelValue"
      :disabled="disabled"
      @change="toggle"
      class="sr-only peer"
    />
    <div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
    <span v-if="label" class="ms-3 text-sm font-medium text-gray-900">{{ label }}</span>
  </label>
</template>