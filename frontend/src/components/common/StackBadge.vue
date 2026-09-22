<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { Icon } from '@iconify/vue';
import { getStackColor, readableTextColor } from '@/utils/colors';
import { useConfig } from '@/composables/useConfig';
import { useStackIcons } from '@/composables/useStackIcons';
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

const { config } = useConfig();
const { iconsReady, loadStackIcons, iconFor, colorFor } = useStackIcons();

const colorClass = computed(() => getStackColor(props.name));

const iconsEnabled = computed(() => config.value?.stack_icons_enabled ?? false);

// Depend on `iconsReady` so the icon appears once the set has loaded.
const icon = computed(() => {
  if (!iconsEnabled.value || !iconsReady.value) {
    return undefined;
  }
  return iconFor(props.name);
});

// When the stack has an icon, tint the pill with its brand colour; otherwise
// fall back to the deterministic DaisyUI colour class.
const brandColor = computed(() => {
  if (!iconsEnabled.value || !iconsReady.value) {
    return undefined;
  }
  return colorFor(props.name);
});

const badgeStyle = computed(() =>
  brandColor.value
    ? {
        backgroundColor: brandColor.value,
        borderColor: brandColor.value,
        color: readableTextColor(brandColor.value),
      }
    : undefined
);

onMounted(() => {
  if (iconsEnabled.value) {
    loadStackIcons();
  }
});

function click() {
  if (props.clickable) {
    emit('click');
  }
}
</script>

<template>
  <span
    class="badge badge-sm gap-1.5"
    :class="[brandColor ? '' : colorClass, removable ? 'pr-1' : '']"
    :style="badgeStyle"
  >
    <Icon v-if="icon" :icon="icon" class="w-3.5 h-3.5 shrink-0" />
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
