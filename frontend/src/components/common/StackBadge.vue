<script setup lang="ts">
import { computed } from 'vue';
import { getStackColor } from '@/utils/colors';
import { X } from 'lucide-vue-next';

const props = defineProps<{
  name: string;
  clickable?: boolean;
  removable?: boolean;
}>();

const emit = defineEmits<{
  remove: [];
  click: [];
}>();

const colorClass = computed(() => getStackColor(props.name));

function click() {
  if (props.clickable) {
    emit('click');
  }
}
</script>

<template>
  <span
    class="badge badge-sm gap-1.5"
    :class="removable ? `${colorClass} pr-1` : `${colorClass}`"
  >
    <p @click="click" :class="clickable ? 'cursor-pointer' : ''">
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
