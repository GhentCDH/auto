<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();
const searchQuery = ref('');

const navItems = [
  { name: 'Dashboard', path: '/' },
  { name: 'Applications', path: '/applications' },
  { name: 'Hosts', path: '/hosts' },
  { name: 'Domains', path: '/domains' },
  { name: 'People', path: '/people' },
  { name: 'Shares', path: '/shares' },
];

function handleSearch() {
  if (searchQuery.value.trim()) {
    router.push({ path: '/search', query: { q: searchQuery.value } });
  }
}
</script>

<template>
  <div class="navbar bg-base-200 shadow-sm">
    <div class="navbar-start">
      <div class="dropdown">
        <div tabindex="0" role="button" class="btn btn-ghost lg:hidden">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 6h16M4 12h8m-8 6h16"
            />
          </svg>
        </div>
        <ul
          tabindex="0"
          class="menu menu-md dropdown-content bg-base-100 rounded-box z-1 mt-3 w-52 p-2 shadow"
        >
          <li v-for="item in navItems" :key="item.path">
            <router-link :to="item.path">{{ item.name }}</router-link>
          </li>
        </ul>
      </div>
      <router-link to="/" class="btn btn-ghost text-xl">Infra Docs</router-link>
    </div>
    <div class="navbar-center hidden lg:flex">
      <ul class="menu menu-horizontal px-1">
        <li v-for="item in navItems" :key="item.path">
          <router-link :to="item.path" class="btn btn-ghost btn-md">{{
            item.name
          }}</router-link>
        </li>
      </ul>
    </div>
    <div class="navbar-end">
      <form @submit.prevent="handleSearch" class="form-control">
        <div class="input-group">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search..."
            class="input input-bordered input-sm md:input-md w-40 md:w-64"
          />
          <button type="submit" class="btn input-sm md:btn-md btn-square">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-4 w-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              />
            </svg>
          </button>
        </div>
      </form>
    </div>
  </div>
</template>
