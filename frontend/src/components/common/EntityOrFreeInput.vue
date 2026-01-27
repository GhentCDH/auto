<script setup lang="ts">
import { ref, watch } from 'vue';
import EntitySelector from './EntitySelector.vue';
import { Check, X } from 'lucide-vue-next';

type InputType = 'entity' | 'free';

const props = withDefaults(
  defineProps<{
    label?: string;
    entityLabel?: string;
    freeLabel?: string;
    freePlaceholder?: string;
    entityTitle: string;
    fetchFn: (params: {
      search?: string;
    }) => Promise<{ data: Array<{ id: string; name: string }> }>;
    // Initial values
    initialEntityId?: string | null;
    initialEntityName?: string | null;
    initialFreeValue?: string | null;
  }>(),
  {
    label: '',
    entityLabel: 'Select',
    freeLabel: 'Free Entry',
    freePlaceholder: 'Enter value',
  }
);

const emit = defineEmits<{
  'update:entityId': [value: string | null];
  'update:freeValue': [value: string | null];
}>();

// Determine initial type based on provided values
const initialType: InputType = props.initialEntityId ? 'entity' : 'free';

const inputType = ref<InputType>(initialType);
const freeValue = ref(props.initialFreeValue || '');
const selectedEntityId = ref<string | null>(props.initialEntityId || null);
const selectedEntityName = ref<string | null>(props.initialEntityName || null);

function handleEntitySelect(entity: { id: string; name: string }) {
  selectedEntityId.value = entity.id;
  selectedEntityName.value = entity.name;
}

function clearEntity() {
  selectedEntityId.value = null;
  selectedEntityName.value = null;
}

// Emit changes
watch(inputType, (type) => {
  if (type === 'entity') {
    emit('update:freeValue', null);
    emit('update:entityId', selectedEntityId.value);
  } else {
    emit('update:entityId', null);
    emit('update:freeValue', freeValue.value || null);
  }
});

watch(selectedEntityId, (id) => {
  if (inputType.value === 'entity') {
    emit('update:entityId', id);
  }
});

watch(freeValue, (val) => {
  if (inputType.value === 'free') {
    emit('update:freeValue', val || null);
  }
});
</script>

<template>
  <fieldset class="fieldset">
    <legend v-if="label" class="fieldset-legend">{{ label }}</legend>
    <select v-model="inputType" class="select w-full">
      <option value="entity">{{ entityLabel }}</option>
      <option value="free">{{ freeLabel }}</option>
    </select>

    <input
      v-if="inputType === 'free'"
      v-model="freeValue"
      type="text"
      class="input w-full mt-2"
      :placeholder="freePlaceholder"
    />

    <EntitySelector
      v-else-if="selectedEntityName === null"
      :title="entityTitle"
      :fetchFn="fetchFn"
      :allowCreate="false"
      @select="handleEntitySelect"
      @cancel="inputType = 'free'"
      class="mt-2"
    />

    <div
      v-else
      class="flex items-center justify-between mt-2 p-2 bg-base-200 rounded"
    >
      <div class="flex items-center gap-2">
        <Check class="h-4 w-4 text-success" />
        {{ selectedEntityName }}
      </div>
      <X class="h-4 w-4 text-error cursor-pointer" @click="clearEntity" />
    </div>
  </fieldset>
</template>
