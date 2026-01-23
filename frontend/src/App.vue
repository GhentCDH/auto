<script setup lang="ts">
import NavBar from './components/layout/NavBar.vue';
</script>

<template>
  <div class="min-h-screen bg-base-100">
    <NavBar />
    <main class="container mx-auto max-w-7xl">
      <router-view />
    </main>
  </div>
</template>
