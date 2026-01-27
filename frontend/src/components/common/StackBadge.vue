<script setup lang="ts">
import { computed } from 'vue';
import { getStackColor } from '@/utils/colors';
import { X } from 'lucide-vue-next';

const props = defineProps<{
  name: string;
  removable?: boolean;
}>();

const emit = defineEmits<{
  remove: [];
}>();

const colorClass = computed(() => getStackColor(props.name));
</script>

<template>
  <span class="badge badge-sm gap-1" :class="colorClass">
    {{ name }}
    <button
      v-if="removable"
      class="hover:opacity-70"
      @click.stop="emit('remove')"
    >
      <X class="w-3 h-3" />
    </button>
  </span>
</template>
