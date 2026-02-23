<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import type {
  Application,
  CreateApplication,
  ImageRef,
  UpdateApplication,
} from '@/types';
import { environments } from '@/values';
import { Plus, Trash2 } from 'lucide-vue-next';

const props = defineProps<{
  application?: Application;
  duplicateApplication?: Application;
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateApplication | UpdateApplication];
  cancel: [];
}>();

const form = ref<CreateApplication>({
  name: props.initialName || '',
  description: '',
  repository_url: '',
  environment: 'prd',
  url: '',
  status: 'active',
});

const imageRefList = ref<ImageRef[]>([]);

watch(
  () => props.application,
  (app) => {
    if (app) {
      form.value = {
        name: app.name,
        description: app.description || '',
        repository_url: app.repository_url || '',
        environment: app.environment,
        url: app.url || '',
        status: app.status,
      };
      imageRefList.value = app.image_refs ? JSON.parse(app.image_refs) : [];
    }
  },
  { immediate: true }
);

watch(
  () => props.duplicateApplication,
  (app) => {
    if (app) {
      form.value = {
        name: `${app.name} (copy)`,
        description: app.description || '',
        repository_url: app.repository_url || '',
        environment: app.environment,
        url: app.url || '',
        status: app.status,
      };
      imageRefList.value = app.image_refs ? JSON.parse(app.image_refs) : [];
    }
  },
  { immediate: true }
);

function addImageRef() {
  imageRefList.value.push({ url: '', alias: '' });
}

function removeImageRef(index: number) {
  imageRefList.value.splice(index, 1);
}

function handleSubmit() {
  const refs = imageRefList.value.filter((r) => r.url.trim());
  emit('submit', {
    ...form.value,
    image_refs: JSON.stringify(
      refs.map((r) => ({
        url: r.url,
        ...(r.alias ? { alias: r.alias } : {}),
      }))
    ),
  });
}

const nameInput = ref<HTMLInputElement>();

onMounted(() => {
  nameInput.value?.focus();
});
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <fieldset class="fieldset">
      <legend class="fieldset-legend">Name *</legend>
      <input
        v-model="form.name"
        ref="nameInput"
        type="text"
        class="input w-full"
        required
      />
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Description</legend>
      <textarea v-model="form.description" class="textarea w-full" rows="3" />
      <div class="label">optional</div>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Repository URL</legend>
      <input
        v-model="form.repository_url"
        type="url"
        class="input w-full"
        placeholder="https://github.com/..."
      />
      <div class="label">optional</div>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">URL</legend>
      <input
        v-model="form.url"
        type="url"
        class="input w-full"
        placeholder="https://..."
      />
      <div class="label">optional - main URL to access the application</div>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Environment</legend>
      <select v-model="form.environment" class="select w-full">
        <option
          v-for="(label, value) in environments"
          :key="value"
          :value="value"
        >
          {{ label }}
        </option>
      </select>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Status</legend>
      <select v-model="form.status" class="select w-full">
        <option value="active">Active</option>
        <option value="inactive">Inactive</option>
        <option value="deprecated">Deprecated</option>
        <option value="archived">Archived</option>
      </select>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Image References</legend>
      <div class="space-y-2">
        <div
          v-for="(imageRef, index) in imageRefList"
          :key="index"
          class="flex gap-2 items-center"
        >
          <input
            v-model="imageRef.url"
            type="text"
            class="input input-sm flex-1"
            placeholder="docker.io/owner/image"
          />
          <input
            v-model="imageRef.alias"
            type="text"
            class="input input-sm w-32"
            placeholder="alias (opt.)"
          />
          <button
            type="button"
            class="btn btn-ghost btn-sm text-error"
            @click="removeImageRef(index)"
          >
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
        <button type="button" class="btn btn-ghost btn-sm" @click="addImageRef">
          <Plus class="w-4 h-4" /> Add image
        </button>
      </div>
      <div class="label">optional - Docker Hub, GHCR, etc.</div>
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn" @click="emit('cancel')">Cancel</button>
      <button type="submit" class="btn btn-primary">
        {{ application ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
