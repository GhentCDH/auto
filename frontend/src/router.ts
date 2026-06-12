import { createRouter, createWebHistory } from 'vue-router';
import { initAuth, useAuth } from './composables/useAuth';
import { loadConfig } from './composables/useConfig';

const routes = [
  // Public auth pages
  {
    path: '/login',
    component: () => import('./views/auth/LoginView.vue'),
    meta: { public: true },
  },
  {
    path: '/set-password/:token',
    component: () => import('./views/auth/SetPasswordView.vue'),
    meta: { public: true },
  },
  {
    path: '/link-account',
    component: () => import('./views/auth/LinkAccountView.vue'),
    meta: { public: true },
  },
  // Admin
  {
    path: '/admin/users',
    component: () => import('./views/admin/UserManagementView.vue'),
    meta: { requiresAdmin: true },
  },
  {
    path: '/',
    component: () => import('./views/dashboard/DashboardView.vue'),
  },
  // Applications
  {
    path: '/applications',
    component: () => import('./views/applications/ApplicationList.vue'),
  },
  {
    path: '/applications/:id',
    component: () => import('./views/applications/ApplicationDetail.vue'),
  },
  // Services
  {
    path: '/services',
    component: () => import('./views/services/ServiceList.vue'),
  },
  {
    path: '/services/:id',
    component: () => import('./views/services/ServiceDetail.vue'),
  },
  // Infra
  {
    path: '/infra',
    component: () => import('./views/infra/InfraList.vue'),
  },
  {
    path: '/infra/:id',
    component: () => import('./views/infra/InfraDetail.vue'),
  },
  // Domains
  {
    path: '/domains',
    component: () => import('./views/domains/DomainList.vue'),
  },
  {
    path: '/domains/:id',
    component: () => import('./views/domains/DomainDetail.vue'),
  },
  // People
  {
    path: '/people',
    component: () => import('./views/people/PersonList.vue'),
  },
  {
    path: '/people/:id',
    component: () => import('./views/people/PersonDetail.vue'),
  },
  // Network Shares
  {
    path: '/shares',
    component: () => import('./views/shares/ShareList.vue'),
  },
  {
    path: '/shares/:id',
    component: () => import('./views/shares/ShareDetail.vue'),
  },
  // Stack
  {
    path: '/stack',
    component: () => import('./views/stack/StackList.vue'),
  },
  {
    path: '/stack/:id',
    component: () => import('./views/stack/StackDetail.vue'),
  },
  // Healthchecks
  {
    path: '/healthchecks',
    component: () => import('./views/healthchecks/HealthcheckList.vue'),
  },
  {
    path: '/healthchecks/:id',
    component: () => import('./views/healthchecks/HealthcheckDetail.vue'),
  },
  // Org graph
  {
    path: '/graph',
    component: () => import('./views/graph/OrgGraphView.vue'),
  },
  // Search
  {
    path: '/search',
    component: () => import('./views/search/SearchView.vue'),
  },
  // Resolve by UUID or UUID prefix (min 8 hex chars)
  {
    path: '/:id([0-9a-f]{8,})',
    component: () => import('./views/ResolveView.vue'),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

// Auth guard: resolve config + session once (shared promises), then gate routes.
router.beforeEach(async (to) => {
  await loadConfig().catch(() => undefined);
  const { authEnabled, isAuthenticated, isAdmin } = useAuth();

  // Open mode (no login method enabled): no gating at all.
  if (!authEnabled.value) return true;

  await initAuth();

  if (!to.meta.public && !isAuthenticated.value) {
    return { path: '/login' };
  }
  if (to.path === '/login' && isAuthenticated.value) {
    return { path: '/' };
  }
  if (to.meta.requiresAdmin && !isAdmin.value) {
    return { path: '/' };
  }
  return true;
});

export default router;
