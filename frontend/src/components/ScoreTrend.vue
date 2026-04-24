<script setup lang="ts">
import { computed } from 'vue';
import VueApexCharts from 'vue3-apexcharts';

const props = defineProps<{
  scores: { date: string; avg_score: number }[];
}>();

const series = computed(() => [{
  data: props.scores.map(s => s.avg_score),
}]);

const chartOptions = computed(() => ({
  chart: { sparkline: { enabled: true } },
  stroke: { curve: 'smooth' as const, width: 2 },
  colors: ['#22c55e'],
  tooltip: { enabled: true },
  yaxis: { min: 0, max: 100 },
}));
</script>

<template>
  <div class="score-trend" aria-label="7-day score trend">
    <VueApexCharts type="line" width="100%" height="40" :options="chartOptions" :series="series" />
  </div>
</template>

<style scoped>
.score-trend {
  width: 100%;
}
</style>