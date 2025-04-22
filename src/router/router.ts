import {createRouter, createWebHistory} from 'vue-router'

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: [
        {
            path: '/',
            name: 'home',
            component: () => import('../views/HomeView.vue'),
            children: [
                {
                    path: '',
                    name: 'Home',
                    component: () => import('../views/Dashboard.vue')
                },
                {
                    path: '/sensor/:fieldKey',
                    name: 'SensorDetail',
                    component: () => import('../components/SensorChart.vue'),
                    // send fieldKey to SensorChart.vue
                    props: true
                },
                {
                    path: '/settings',
                    name: 'Settings',
                    component: () => import('../views/Settings.vue')
                }
            ]
        },

    ]
})

export default router