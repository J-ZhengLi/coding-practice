import { defineStore } from 'pinia';
import { ref } from 'vue';
import {
  fetchMaterials as fetchMaterialsApi,
  triggerFetchMaterials as triggerFetchApi,
  refreshMaterial as refreshMaterialApi,
  deleteMaterial as deleteMaterialApi,
  type Material,
} from '../api/materials';

export const useMaterialStore = defineStore('material', () => {
  // State
  const materials = ref<Material[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Actions
  const fetchMaterials = async (language?: string, difficulty?: string) => {
    loading.value = true;
    error.value = null;

    try {
      materials.value = await fetchMaterialsApi(language, difficulty);
    } catch (err: any) {
      error.value = err.response?.data?.error || err.message || 'Failed to fetch materials';
    } finally {
      loading.value = false;
    }
  };

  const triggerFetch = async (language: string, difficulty: string) => {
    loading.value = true;
    error.value = null;

    try {
      const fetched = await triggerFetchApi(language, difficulty);
      materials.value = [...materials.value, ...fetched];
    } catch (err: any) {
      error.value = err.response?.data?.error || err.message || 'Failed to fetch materials';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const refreshMaterial = async (id: number) => {
    loading.value = true;
    error.value = null;

    try {
      const refreshed = await refreshMaterialApi(id);
      const index = materials.value.findIndex((m) => m.id === id);
      if (index !== -1) {
        materials.value[index] = refreshed;
      }
    } catch (err: any) {
      error.value = err.response?.data?.error || err.message || 'Failed to refresh material';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const deleteMaterial = async (id: number) => {
    loading.value = true;
    error.value = null;

    try {
      await deleteMaterialApi(id);
      materials.value = materials.value.filter((m) => m.id !== id);
    } catch (err: any) {
      error.value = err.response?.data?.error || err.message || 'Failed to delete material';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  // Getters
  const materialCountByLanguage = (language: string): number => {
    return materials.value.filter((m) => m.language === language).length;
  };

  return {
    // State
    materials,
    loading,
    error,
    // Actions
    fetchMaterials,
    triggerFetch,
    refreshMaterial,
    deleteMaterial,
    // Getters
    materialCountByLanguage,
  };
});