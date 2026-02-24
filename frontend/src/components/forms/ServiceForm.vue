<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import type { Service, CreateService, ImageRef, UpdateService } from '@/types';
import { environments } from '@/values';
import { Plus, Trash2 } from 'lucide-vue-next';

const props = defineProps<{
  service?: Service;
  duplicateService?: Service;
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateService | UpdateService];
  cancel: [];
}>();

const form = ref<CreateService>({
  name: props.initialName || '',
  description: '',
  repository_url: '',
  environment: 'prd',
  status: 'active',
  outline_url: '',
});

const imageRefList = ref<ImageRef[]>([]);

watch(
  () => props.service,
  (svc) => {
    if (svc) {
      form.value = {
        name: svc.name,
        description: svc.description || '',
        repository_url: svc.repository_url || '',
        environment: svc.environment,
        status: svc.status,
        outline_url: svc.outline_url || '',
      };
      imageRefList.value = svc.image_refs ? JSON.parse(svc.image_refs) : [];
    }
  },
  { immediate: true }
);

watch(
  () => props.duplicateService,
  (svc) => {
    if (svc) {
      form.value = {
        name: `${svc.name} (copy)`,
        description: svc.description || '',
        repository_url: svc.repository_url || '',
        environment: svc.environment,
        status: svc.status,
        outline_url: svc.outline_url || '',
      };
      imageRefList.value = svc.image_refs ? JSON.parse(svc.image_refs) : [];
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
      <legend class="fieldset-legend">Outline Document URL</legend>
      <input
        v-model="form.outline_url"
        type="url"
        class="input w-full"
        placeholder="https://docs.example.com/doc/..."
      />
      <div class="label">
        optional - link to the Outline wiki page for this service
      </div>
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
          v-for="(imgRef, index) in imageRefList"
          :key="index"
          class="flex gap-2 items-center"
        >
          <input
            v-model="imgRef.url"
            type="text"
            class="input input-sm flex-1"
            placeholder="docker.io/owner/image"
          />
          <input
            v-model="imgRef.alias"
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
        {{ service ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
