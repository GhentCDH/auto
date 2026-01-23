<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  page: number;
  totalPages: number;
  total: number;
}>();

const emit = defineEmits<{
  'update:page': [page: number];
}>();

const pages = computed(() => {
  const result: (number | string)[] = [];
  const current = props.page;
  const total = props.totalPages;

  if (total <= 7) {
    for (let i = 1; i <= total; i++) {
      result.push(i);
    }
  } else {
    result.push(1);
    if (current > 3) {
      result.push('...');
    }
    for (
      let i = Math.max(2, current - 1);
      i <= Math.min(total - 1, current + 1);
      i++
    ) {
      result.push(i);
    }
    if (current < total - 2) {
      result.push('...');
    }
    result.push(total);
  }

  return result;
});

function goToPage(page: number) {
  if (page >= 1 && page <= props.totalPages) {
    emit('update:page', page);
  }
}
</script>

<template>
  <div class="flex items-center justify-between mt-4">
    <div class="text-sm text-base-content/70">Total: {{ total }} items</div>
    <div class="join" v-if="totalPages > 1">
      <button
        class="join-item btn btn-sm"
        :disabled="page === 1"
        @click="goToPage(page - 1)"
      >
        &laquo;
      </button>
      <template v-for="p in pages" :key="p">
        <button
          v-if="typeof p === 'number'"
          class="join-item btn btn-sm"
          :class="{ 'btn-active': p === page }"
          @click="goToPage(p)"
        >
          {{ p }}
        </button>
        <button v-else class="join-item btn btn-sm btn-disabled">
          {{ p }}
        </button>
      </template>
      <button
        class="join-item btn btn-sm"
        :disabled="page === totalPages"
        @click="goToPage(page + 1)"
      >
        &raquo;
      </button>
    </div>
  </div>
</template>
