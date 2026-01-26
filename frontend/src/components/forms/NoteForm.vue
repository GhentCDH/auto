<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Note, CreateNote, UpdateNote } from '@/types';
import { noteTypes } from '@/values';
import MarkdownRenderer from '../common/MarkdownRenderer.vue';
import { Eye, EyeOff } from 'lucide-vue-next';

const props = defineProps<{
  note?: Note;
  entityType: string;
  entityId: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateNote | UpdateNote];
  cancel: [];
}>();

const title = ref(props.note?.title || '');
const content = ref(props.note?.content || '');
const note_type = ref(props.note?.note_type || 'general');
const url = ref(props.note?.url || '');
const is_pinned = ref(props.note?.is_pinned || false);
const showPreview = ref(false);

const isEditing = computed(() => !!props.note);

function handleSubmit() {
  if (isEditing.value) {
    const data: UpdateNote = {
      title: title.value,
      content: content.value || undefined,
      note_type: note_type.value,
      url: url.value || undefined,
      is_pinned: is_pinned.value,
    };
    emit('submit', data);
  } else {
    const data: CreateNote = {
      entity_type: props.entityType,
      entity_id: props.entityId,
      title: title.value,
      content: content.value || undefined,
      note_type: note_type.value,
      url: url.value || undefined,
      is_pinned: is_pinned.value,
    };
    emit('submit', data);
  }
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <fieldset class="fieldset">
      <legend class="fieldset-legend">Title *</legend>
      <input
        v-model="title"
        type="text"
        class="input w-full"
        placeholder="Note title"
        required
      />
    </fieldset>

    <div class="grid grid-cols-2 gap-4">
      <fieldset class="fieldset">
        <legend class="fieldset-legend">Type</legend>
        <select v-model="note_type" class="select w-full">
          <option v-for="(visual, value) in noteTypes" :key="value" :value="value">
            {{ visual }}
          </option>
        </select>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Pinned</legend>
        <label class="flex items-center gap-3 cursor-pointer h-12">
          <input
            v-model="is_pinned"
            type="checkbox"
            class="checkbox checkbox-primary"
          />
          <span>Pin to top</span>
        </label>
      </fieldset>
    </div>

    <fieldset v-if="note_type === 'link'" class="fieldset">
      <legend class="fieldset-legend">URL</legend>
      <input
        v-model="url"
        type="url"
        class="input w-full"
        placeholder="https://..."
      />
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend flex items-center justify-between w-full">
        <span>Content</span>
        <button
          type="button"
          class="btn btn-ghost btn-xs gap-1"
          @click="showPreview = !showPreview"
        >
          <Eye v-if="!showPreview" class="h-3 w-3" />
          <EyeOff v-else class="h-3 w-3" />
          {{ showPreview ? 'Edit' : 'Preview' }}
        </button>
      </legend>
      <div v-if="showPreview" class="min-h-[150px] p-3 bg-base-100 rounded-lg border border-base-300">
        <MarkdownRenderer v-if="content" :content="content" />
        <span v-else class="text-base-content/50 italic">No content</span>
      </div>
      <textarea
        v-else
        v-model="content"
        class="textarea w-full font-mono text-sm"
        rows="8"
        placeholder="Write your note here... Markdown is supported!"
      />
      <div class="label text-xs">
        Supports Markdown: **bold**, *italic*, `code`, [links](url), lists, etc.
      </div>
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn btn-ghost" @click="emit('cancel')">
        Cancel
      </button>
      <button type="submit" class="btn btn-primary" :disabled="!title.trim()">
        {{ isEditing ? 'Update Note' : 'Create Note' }}
      </button>
    </div>
  </form>
</template>
