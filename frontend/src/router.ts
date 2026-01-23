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
  // Hosts
  {
    path: '/hosts',
    component: () => import('./views/hosts/HostList.vue'),
  },
  {
    path: '/hosts/:id',
    component: () => import('./views/hosts/HostDetail.vue'),
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
  // Clients
  {
    path: '/clients',
    component: () => import('./views/clients/ClientList.vue'),
  },
  {
    path: '/clients/:id',
    component: () => import('./views/clients/ClientDetail.vue'),
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
