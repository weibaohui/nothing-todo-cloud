import { test, expect } from '@playwright/test';

const BASE_URL = 'http://localhost:8089';

test.describe('ntd-cloud Frontend Tests', () => {
  test('homepage loads correctly', async ({ page }) => {
    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: 10000 });
    await expect(page).toHaveTitle(/ntd-cloud/);
  });

  test('health endpoint returns ok', async ({ request }) => {
    const response = await request.get(`${BASE_URL}/health`);
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.status).toBe('ok');
  });

  test('login page accessible via SPA route', async ({ page }) => {
    await page.goto(`${BASE_URL}/login`, { waitUntil: 'domcontentloaded', timeout: 10000 });
    // SPA路由返回index.html，body应该是HTML文档
    const content = await page.content();
    expect(content).toContain('html');
  });

  test('dashboard page accessible via SPA route', async ({ page }) => {
    await page.goto(`${BASE_URL}/dashboard`, { waitUntil: 'domcontentloaded', timeout: 10000 });
    // SPA路由返回index.html
    const content = await page.content();
    expect(content).toContain('html');
  });
});

test.describe('Auth API Tests', () => {
  test('register new user', async ({ request }) => {
    const timestamp = Date.now();
    const response = await request.post(`${BASE_URL}/api/auth/register`, {
      data: {
        email: `test${timestamp}@example.com`,
        password: '123456'
      }
    });
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.success).toBe(true);
    expect(body.token).toBeDefined();
  });

  test('login with valid credentials', async ({ request }) => {
    // First register
    const timestamp = Date.now();
    await request.post(`${BASE_URL}/api/auth/register`, {
      data: {
        email: `logintest${timestamp}@example.com`,
        password: '123456'
      }
    });

    // Then login
    const response = await request.post(`${BASE_URL}/api/auth/login`, {
      data: {
        email: `logintest${timestamp}@example.com`,
        password: '123456'
      }
    });
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.success).toBe(true);
    expect(body.token).toBeDefined();
  });

  test('login with invalid credentials fails', async ({ request }) => {
    const response = await request.post(`${BASE_URL}/api/auth/login`, {
      data: {
        email: 'nonexistent@example.com',
        password: 'wrongpassword'
      }
    });
    expect(response.status()).toBe(401);
  });
});

test.describe('Protected API Tests', () => {
  let authToken: string;

  test.beforeAll(async ({ request }) => {
    const timestamp = Date.now();
    // Register a user
    const regResponse = await request.post(`${BASE_URL}/api/auth/register`, {
      data: {
        email: `prottest${timestamp}@example.com`,
        password: '123456'
      }
    });
    const regBody = await regResponse.json();
    authToken = regBody.token;
  });

  test('get devices without token fails', async ({ request }) => {
    const response = await request.get(`${BASE_URL}/api/devices`);
    expect(response.status()).toBe(401);
  });

  test('get devices with token succeeds', async ({ request }) => {
    const response = await request.get(`${BASE_URL}/api/devices`, {
      headers: { Authorization: `Bearer ${authToken}` }
    });
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(Array.isArray(body)).toBe(true);
  });

  test('create device', async ({ request }) => {
    const response = await request.post(`${BASE_URL}/api/devices`, {
      headers: { Authorization: `Bearer ${authToken}` },
      data: { device_name: 'Test Device' }
    });
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.id).toBeDefined();
    expect(body.device_name).toBe('Test Device');
  });

  test('list devices', async ({ request }) => {
    const response = await request.get(`${BASE_URL}/api/devices`, {
      headers: { Authorization: `Bearer ${authToken}` }
    });
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(Array.isArray(body)).toBe(true);
    expect(body.length).toBeGreaterThan(0);
  });

  test('create API token', async ({ request }) => {
    const response = await request.post(`${BASE_URL}/api/tokens`, {
      headers: { Authorization: `Bearer ${authToken}` },
      data: { name: 'Test Token' }
    });
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.id).toBeDefined();
    expect(body.name).toBe('Test Token');
    expect(body.token).toBeDefined(); // Only available on creation
  });

  test('admin stats', async ({ request }) => {
    const response = await request.get(`${BASE_URL}/api/admin/stats`, {
      headers: { Authorization: `Bearer ${authToken}` }
    });
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.total_users).toBeDefined();
    expect(body.total_devices).toBeDefined();
  });
});
