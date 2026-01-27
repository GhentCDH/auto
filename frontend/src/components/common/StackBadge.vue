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
  <span
    class="badge badge-sm gap-1.5"
    :class="removable ? `${colorClass} pr-1` : `${colorClass}`"
  >
    <p>
      {{ name }}
    </p>
    <button
      v-if="removable"
      class="hover:opacity-70 cursor-pointer"
      @click.stop="emit('remove')"
    >
      <X class="w-3 h-3 mt-px" stroke-width="3px" />
    </button>
  </span>
</template>
