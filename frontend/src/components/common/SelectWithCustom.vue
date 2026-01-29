<script setup lang="ts">
import { computed, ref, watch } from 'vue';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    options: Record<string, string>;
    label?: string;
    allowCustom?: boolean;
    customPlaceholder?: string;
  }>(),
  {
    label: '',
    allowCustom: false,
    customPlaceholder: 'Enter custom value',
  }
);

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const isCustomValue = (value: string) =>
  value !== '' && !Object.keys(props.options).includes(value);

const selectedOption = ref(
  isCustomValue(props.modelValue) ? 'other' : props.modelValue
);
const customValue = ref(
  isCustomValue(props.modelValue) ? props.modelValue : ''
);

const effectiveValue = computed(() =>
  selectedOption.value === 'other'
    ? customValue.value.toLowerCase()
    : selectedOption.value
);

watch(effectiveValue, (val) => {
  emit('update:modelValue', val);
});

watch(
  () => props.modelValue,
  (val) => {
    // Ignore if we're in "other" mode and this is our own emission
    if (selectedOption.value === 'other' && val === customValue.value) {
      return;
    }
    if (isCustomValue(val)) {
      selectedOption.value = 'other';
      customValue.value = val;
    } else {
      selectedOption.value = val;
    }
  }
);
</script>

<template>
  <fieldset class="fieldset">
    <legend v-if="label" class="fieldset-legend">{{ label }}</legend>
    <select v-model="selectedOption" class="select w-full">
      <option v-for="(visual, value) in options" :key="value" :value="value">
        {{ visual }}
      </option>
      <option v-if="allowCustom" value="other">Other...</option>
    </select>
    <input
      v-if="allowCustom && selectedOption === 'other'"
      v-model="customValue"
      type="text"
      class="input w-full mt-2"
      :placeholder="customPlaceholder"
      required
    />
  </fieldset>
</template>
