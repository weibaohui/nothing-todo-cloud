import axios from 'axios';

const API_BASE = '/api';

const client = axios.create({
  baseURL: API_BASE,
  timeout: 30000,
});

// 请求拦截器：添加 Token
client.interceptors.request.use((config) => {
  const token = localStorage.getItem('ntd_cloud_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// 响应拦截器：处理错误
client.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('ntd_cloud_token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// API 方法
export const auth = {
  login: (email: string, password: string) =>
    client.post('/auth/login', { email, password }),
  register: (email: string, password: string) =>
    client.post('/auth/register', { email, password }),
  logout: () => client.post('/auth/logout'),
};

export const tokens = {
  list: () => client.get('/tokens'),
  create: (name: string) => client.post('/tokens', { name }),
  revoke: (id: number) => client.delete(`/tokens/${id}`),
};

export const devices = {
  list: () => client.get('/devices'),
  register: (name: string) => client.post('/devices', { name }),
  delete: (id: number) => client.delete(`/devices/${id}`),
  updateName: (id: number, name: string) =>
    client.put(`/devices/${id}/name`, { name }),
};

export const admin = {
  stats: () => client.get('/admin/stats'),
  listUsers: () => client.get('/admin/users'),
};

export default client;
