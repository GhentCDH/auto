<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { toast } from 'vue-sonner';
import MascotViewer from '../common/MascotViewer.vue';
import { useAuth } from '../../composables/useAuth';
import { authApi, usersApi } from '../../api';

const router = useRouter();
const searchQuery = ref('');

const { user, isAdmin, isAuthenticated, logout } = useAuth();

// Pending password-reset requests, surfaced as a badge for admins.
const pendingResets = ref(0);
async function refreshPendingResets() {
  if (!isAdmin.value) return;
  try {
    pendingResets.value = (await usersApi.resetRequests()).length;
  } catch {
    // best-effort badge
  }
}

async function handleLogout() {
  await logout();
  router.push('/login');
}

// DaisyUI dropdowns are :focus-driven; blurring the focused element closes them.
function closeUserMenu() {
  (document.activeElement as HTMLElement | null)?.blur();
}

// Change-password modal
const pwModal = ref<HTMLDialogElement>();
const currentPw = ref('');
const newPw = ref('');
const pwSaving = ref(false);
function openPwModal() {
  currentPw.value = '';
  newPw.value = '';
  pwModal.value?.showModal();
}
async function submitChangePassword() {
  pwSaving.value = true;
  try {
    await authApi.changePassword(currentPw.value, newPw.value);
    toast.success('Password changed');
    pwModal.value?.close();
  } catch (e: any) {
    toast.error(e.message || 'Could not change password');
  } finally {
    pwSaving.value = false;
  }
}

const windowWidth = ref(window.innerWidth);
const windowHeight = ref(window.innerHeight);

function updateWindowSize() {
  windowWidth.value = window.innerWidth;
  windowHeight.value = window.innerHeight;
}

onMounted(() => window.addEventListener('resize', updateWindowSize));
onUnmounted(() => window.removeEventListener('resize', updateWindowSize));

// md breakpoint = 768px (Tailwind default)
const isMdUp = computed(() => windowWidth.value >= 768);

const proximityRadius = computed(() => {
  const diagonal = Math.sqrt(windowWidth.value ** 2 + windowHeight.value ** 2);
  return diagonal * 0.8;
});

const navItems = [
  { name: 'Dashboard', path: '/' },
  { name: 'Applications', path: '/applications' },
  { name: 'Services', path: '/services' },
  { name: 'Infra', path: '/infra' },
  { name: 'Domains', path: '/domains' },
  { name: 'People', path: '/people' },
  { name: 'Storage', path: '/shares' },
  { name: 'Stack', path: '/stack' },
  { name: 'Healthchecks', path: '/healthchecks' },
  { name: 'Graph', path: '/graph' },
];

const searchInput = ref<HTMLInputElement>();
const searchOpen = ref(false);

// Input visible inline from md up; on sm only once opened via the trigger.
const showInput = computed(() => isMdUp.value || searchOpen.value);
// Icon-only trigger shown on sm while the input is collapsed.
const showTrigger = computed(() => !isMdUp.value && !searchOpen.value);

async function openSearch() {
  searchOpen.value = true;
  await nextTick();
  searchInput.value?.focus();
}

function closeSearch() {
  searchOpen.value = false;
}

function onSearchBlur() {
  // Collapse only on small screens, and keep open if a query is typed.
  if (!isMdUp.value && !searchQuery.value.trim()) closeSearch();
}

function handleSearch() {
  if (searchQuery.value.trim()) {
    router.push({ path: '/search', query: { q: searchQuery.value } });
  }
}

// Keyboard shortcuts
function handleGlobalKeydown(e: KeyboardEvent) {
  if (
    e.target instanceof HTMLInputElement ||
    e.target instanceof HTMLTextAreaElement ||
    e.target instanceof HTMLSelectElement
  )
    return;
  if (e.key === '/') {
    e.preventDefault();
    // On sm the input is collapsed — open it before focusing.
    if (!isMdUp.value) openSearch();
    else searchInput.value?.focus();
  }
}

onMounted(() => document.addEventListener('keydown', handleGlobalKeydown));
onUnmounted(() => document.removeEventListener('keydown', handleGlobalKeydown));

onMounted(refreshPendingResets);
</script>

<template>
  <div class="navbar bg-base-200 shadow-sm">
    <div class="navbar-start">
      <div class="dropdown">
        <div tabindex="0" role="button" class="btn px-2 btn-ghost xl:hidden">
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
      <router-link
        to="/"
        class="btn btn-ghost text-2xl inline-flex items-center gap-3 wallefont font-black"
      >
        <Suspense>
          <span class="hidden md:block mt-2"
            ><MascotViewer :size="40" :proximity-radius="proximityRadius"
          /></span>
          <template #fallback>
            <span class="loading loading-spinner loading-xs"></span>
          </template>
        </Suspense>
        AUTO
      </router-link>
    </div>
    <div class="navbar-center hidden xl:flex">
      <ul class="menu menu-horizontal px-1">
        <li v-for="item in navItems" :key="item.path">
          <router-link :to="item.path" class="btn btn-ghost btn-md">{{
            item.name
          }}</router-link>
        </li>
      </ul>
    </div>
    <div class="navbar-end relative">
      <!-- sm: icon-only trigger that expands the search input -->
      <button
        v-if="showTrigger"
        type="button"
        @click="openSearch"
        aria-label="Open search"
        class="btn btn-sm btn-square"
      >
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

      <!-- search input: inline from md up, overlay on sm once opened -->
      <form
        v-show="showInput"
        @submit.prevent="handleSearch"
        class="form-control"
        :class="
          !isMdUp
            ? 'absolute inset-y-0 right-0 z-20 flex items-center bg-base-200 pl-2'
            : ''
        "
      >
        <div class="input-group flex">
          <input
            ref="searchInput"
            v-model="searchQuery"
            type="text"
            placeholder="Search..."
            class="input input-bordered input-sm md:input-md w-44 md:w-64"
            @blur="onSearchBlur"
            @keydown.esc="closeSearch"
          />
          <button type="submit" class="btn btn-sm md:btn-md btn-square">
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

      <!-- User menu -->
      <div v-if="isAuthenticated" class="dropdown dropdown-end ml-2">
        <div
          tabindex="0"
          role="button"
          class="btn btn-ghost btn-circle avatar placeholder indicator"
        >
          <span
            v-if="isAdmin && pendingResets > 0"
            class="indicator-item badge badge-error badge-xs"
            >{{ pendingResets }}</span
          >
          <div
            class="bg-neutral text-neutral-content w-9 rounded-full flex items-center justify-center"
          >
            <span class="text-sm uppercase">{{
              user?.username?.charAt(0) ?? '?'
            }}</span>
          </div>
        </div>
        <ul
          tabindex="0"
          class="menu menu-sm dropdown-content bg-base-100 rounded-box z-10 mt-3 w-56 p-2 shadow"
          @click="closeUserMenu"
        >
          <li class="menu-title flex flex-row items-center justify-between">
            <span class="truncate">{{ user?.username }}</span>
            <span class="badge badge-ghost badge-sm">{{ user?.role }}</span>
          </li>
          <li><button @click="openPwModal">Change password</button></li>
          <li v-if="isAdmin">
            <router-link to="/admin/users" class="flex justify-between">
              <span>User management</span>
              <span
                v-if="pendingResets > 0"
                class="badge badge-error badge-sm"
                >{{ pendingResets }}</span
              >
            </router-link>
          </li>
          <li><button @click="handleLogout">Log out</button></li>
        </ul>
      </div>
    </div>
  </div>

  <!-- Change-password modal -->
  <dialog ref="pwModal" class="modal">
    <div class="modal-box">
      <h3 class="text-lg font-bold">Change password</h3>
      <form @submit.prevent="submitChangePassword" class="mt-4 space-y-3">
        <input
          v-model="currentPw"
          type="password"
          required
          placeholder="Current password"
          class="input input-bordered w-full"
          autocomplete="current-password"
        />
        <input
          v-model="newPw"
          type="password"
          required
          minlength="8"
          placeholder="New password"
          class="input input-bordered w-full"
          autocomplete="new-password"
        />
        <div class="modal-action">
          <button
            type="button"
            class="btn btn-ghost"
            @click="pwModal?.close()"
          >
            Cancel
          </button>
          <button type="submit" class="btn btn-primary" :disabled="pwSaving">
            Save
          </button>
        </div>
      </form>
    </div>
    <form method="dialog" class="modal-backdrop">
      <button>close</button>
    </form>
  </dialog>
</template>
