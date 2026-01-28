import { createRouter, createWebHistory } from 'vue-router';

const routes = [
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
  // Search
  {
    path: '/search',
    component: () => import('./views/search/SearchView.vue'),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
