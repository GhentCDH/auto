import { createRouter, createWebHistory } from 'vue-router';
import MainPage from './views/MainPage.vue';

const routes = [{ path: '/', component: MainPage }];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
