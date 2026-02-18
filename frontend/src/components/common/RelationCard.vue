<script setup lang="ts" generic="T extends { id: string }">
import { Plus, Edit, Link2Off } from 'lucide-vue-next';

defineProps<{
  /** Card title (e.g., "Infrastructure", "Services") */
  title: string;
  /** Items to display */
  items: T[];
  /** Message when no items exist */
  emptyMessage: string;
  /** Show edit buttons */
  showEdit?: boolean;
  /** Show unlink buttons */
  showUnlink?: boolean;
  /** Use table layout instead of list */
  tableLayout?: boolean;
}>();

const emit = defineEmits<{
  /** Add button clicked */
  add: [];
  /** Edit button clicked for an item */
  edit: [item: T];
  /** Unlink button clicked for an item */
  unlink: [item: T];
  /** Item row/element clicked */
  itemClick: [item: T];
}>();

defineSlots<{
  /** Table header row (for tableLayout) */
  tableHeader?: () => unknown;
  /** Table row content (for tableLayout) */
  tableRow?: (props: { item: T }) => unknown;
  /** List item content (for list layout) */
  listItem?: (props: { item: T }) => unknown;
  /** Badge content (for badge layout like stacks) */
  badge?: (props: { item: T }) => unknown;
  /** Custom actions beyond edit/unlink */
  actions?: (props: { item: T }) => unknown;
}>();
</script>

<template>
  <div class="card bg-base-200">
    <div class="card-body">
      <!-- Header -->
      <div class="flex justify-between items-center">
        <h2 class="card-title">{{ title }}</h2>
        <button class="btn btn-sm btn-ghost" @click="emit('add')">
          <Plus class="w-4 h-4" /> Add
        </button>
      </div>

      <!-- Empty state -->
      <div v-if="items.length === 0" class="text-base-content/70">
        {{ emptyMessage }}
      </div>

      <!-- Table layout -->
      <div v-else-if="tableLayout" class="overflow-x-auto">
        <table class="table table-sm">
          <thead>
            <tr>
              <slot name="tableHeader" />
              <th class="w-20"></th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in items"
              :key="item.id"
              class="align-middle"
            >
              <slot name="tableRow" :item="item" />
              <td class="flex justify-end">
                <slot name="actions" :item="item" />
                <button
                  v-if="showEdit"
                  class="btn btn-ghost btn-xs"
                  @click.stop="emit('edit', item)"
                >
                  <Edit class="w-4 h-4" />
                </button>
                <button
                  v-if="showUnlink"
                  class="btn btn-ghost btn-xs text-error"
                  @click.stop="emit('unlink', item)"
                >
                  <Link2Off class="w-4 h-4" />
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Badge layout (for stacks) -->
      <div v-else-if="$slots.badge" class="flex flex-wrap gap-2">
        <template v-for="item in items" :key="item.id">
          <slot name="badge" :item="item" />
        </template>
      </div>

      <!-- List layout -->
      <ul v-else class="space-y-2">
        <li
          v-for="item in items"
          :key="item.id"
          class="flex items-center justify-between"
        >
          <div
            class="flex-1 cursor-pointer"
            @click="emit('itemClick', item)"
          >
            <slot name="listItem" :item="item" />
          </div>
          <div class="flex justify-end">
            <slot name="actions" :item="item" />
            <button
              v-if="showEdit"
              class="btn btn-ghost btn-xs"
              @click.stop="emit('edit', item)"
            >
              <Edit class="w-4 h-4" />
            </button>
            <button
              v-if="showUnlink"
              class="btn btn-ghost btn-xs text-error"
              @click.stop="emit('unlink', item)"
            >
              <Link2Off class="w-4 h-4" />
            </button>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>
