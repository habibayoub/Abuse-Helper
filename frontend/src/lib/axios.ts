import axios from 'axios';

// Create axios instance with default config
const api = axios.create({
    baseURL: '/api',  // This will use the proxy configured in vite.config.ts
    headers: {
        'Content-Type': 'application/json',
    },
});

// Add response interceptor to handle 401 errors
api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            // Optionally redirect to login or handle unauthorized access
            window.location.href = '/login';
        }
        return Promise.reject(error);
    }
);

export default api;
