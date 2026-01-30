<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { Filter } from 'lucide-vue-next';

export interface FilterOption {
  value: string;
  label: string;
}

const props = defineProps<{
  options: FilterOption[];
  modelValue: string | null;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string | null];
}>();

const isOpen = ref(false);
const buttonRef = ref<HTMLButtonElement | null>(null);
const dropdownPosition = ref({ top: 0, left: 0 });

const hasActiveFilter = computed(() => props.modelValue !== null);

function selectOption(value: string | null) {
  emit('update:modelValue', value);
  isOpen.value = false;
}

function updatePosition() {
  if (buttonRef.value) {
    const rect = buttonRef.value.getBoundingClientRect();
    dropdownPosition.value = {
      top: rect.bottom + 4,
      left: rect.right - 160, // 160px = w-40 = 10rem
    };
  }
}

async function toggleDropdown() {
  if (!isOpen.value) {
    updatePosition();
    await nextTick();
  }
  isOpen.value = !isOpen.value;
}

function closeDropdown(event: MouseEvent) {
  const target = event.target as Node;
  if (buttonRef.value && !buttonRef.value.contains(target)) {
    isOpen.value = false;
  }
}

function handleScroll() {
  if (isOpen.value) {
    updatePosition();
  }
}

onMounted(() => {
  document.addEventListener('click', closeDropdown);
  document.addEventListener('scroll', handleScroll, true);
});

onUnmounted(() => {
  document.removeEventListener('click', closeDropdown);
  document.removeEventListener('scroll', handleScroll, true);
});
</script>

<template>
  <button
    ref="buttonRef"
    type="button"
    class="btn btn-ghost btn-xs"
    :class="{ 'text-primary': hasActiveFilter }"
    @click.stop="toggleDropdown"
  >
    <Filter class="h-4 w-4" />
  </button>
  <Teleport to="body">
    <ul
      v-if="isOpen"
      class="fixed z-50 menu p-2 shadow bg-base-200 rounded-box w-40"
      :style="{
        top: `${dropdownPosition.top}px`,
        left: `${dropdownPosition.left}px`,
      }"
    >
      <li>
        <a
          :class="{ active: modelValue === null }"
          @click.stop="selectOption(null)"
        >
          All
        </a>
      </li>
      <li v-for="option in options" :key="option.value">
        <a
          :class="{ active: modelValue === option.value }"
          @click.stop="selectOption(option.value)"
        >
          {{ option.label }}
        </a>
      </li>
    </ul>
  </Teleport>
</template>
