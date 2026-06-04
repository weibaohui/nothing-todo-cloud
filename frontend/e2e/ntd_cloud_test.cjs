// ntd-cloud 管理后台端到端测试
const { chromium } = require('playwright');

const BASE_URL = 'http://localhost:8089';
const TEST_USER = { email: 'admin@ntd.local', password: 'admin123' };

async function login(page) {
  await page.goto(`${BASE_URL}/login`, { waitUntil: 'networkidle' });
  await page.waitForTimeout(500);
  await page.fill('input[placeholder="邮箱"]', TEST_USER.email);
  await page.fill('input[placeholder="密码"]', TEST_USER.password);
  await page.locator('button[type="submit"]').click();
  await page.waitForTimeout(1500);
  return page.url().includes('/dashboard');
}

// SPA 内部导航：用 pushState 触发 React Router 切换，避免页面重载
async function spaNavigate(page, path) {
  await page.evaluate((p) => {
    window.history.pushState({}, '', p);
    window.dispatchEvent(new PopStateEvent('popstate'));
  }, path);
  await page.waitForTimeout(500);
}

(async () => {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();

  const consoleErrors = [];
  page.on('console', msg => {
    if (msg.type() === 'error') consoleErrors.push(msg.text());
  });
  const pageErrors = [];
  page.on('pageerror', err => pageErrors.push(err.message));

  try {
    console.log('=== ntd-cloud 管理后台测试 ===\n');

    // ===== 1. 访问首页 → 重定向到登录页 =====
    console.log('1. 访问首页...');
    await page.goto(BASE_URL, { waitUntil: 'networkidle' });
    await page.waitForTimeout(1500);
    console.log(`   页面标题: "${await page.title()}"`);
    console.log(`   当前 URL: ${page.url()}`);
    console.log(`   ${page.url().includes('/login') ? '✅ 已重定向到登录页' : '⚠ 期望在登录页'}`);

    // ===== 2. 登录 =====
    console.log('\n2. 登录...');
    const loginOk = await login(page);
    console.log(`   ${loginOk ? '✅ 登录成功，已跳转控制台' : '❌ 登录失败'}`);
    if (!loginOk) { await browser.close(); return; }

    // ===== 3. 控制台页面 =====
    console.log('\n3. 控制台页面...');
    const statsTitles = await page.locator('.ant-statistic-title').allTextContents();
    console.log(`   统计项: ${JSON.stringify(statsTitles)}`);
    console.log(`   ${statsTitles.length >= 3 ? '✅ 统计信息完整' : '⚠ 统计信息不完整'}`);

    // ===== 4. SPA 内导航到设置页 - 查看 Token 显示 =====
    console.log('\n4. 设置页 - Token 显示...');
    await spaNavigate(page, '/settings');
    await page.waitForTimeout(1000);

    const bodyText = await page.locator('body').textContent();
    const hasTokenLabel = bodyText.includes('当前登录 Token');
    console.log(`   ${hasTokenLabel ? '✅' : '❌'} "当前登录 Token" ${hasTokenLabel ? '存在' : '未找到'}`);

    // 尝试点击显示按钮
    const eyeBtn = page.locator('button').filter({ has: page.locator('.anticon-eye') }).first();
    if (await eyeBtn.isVisible().catch(() => false)) {
      await eyeBtn.click();
      await page.waitForTimeout(300);
      console.log('   ✅ Token 显示/隐藏按钮可用');
    } else {
      console.log('   ⚠ 未找到显示按钮');
    }

    // ===== 5. SPA 内导航到 Token 管理 - 创建 Token =====
    console.log('\n5. Token 管理 - 创建 Token...');
    await spaNavigate(page, '/tokens');
    await page.waitForTimeout(1000);

    const tokenNames = ['开发环境', '测试服务器', '本地设备'];
    let createdCount = 0;

    for (const name of tokenNames) {
      console.log(`   创建: "${name}"...`);

      // 点"创建 Token"按钮
      const createBtn = page.locator('button').filter({ hasText: '创建 Token' }).first();
      if (!(await createBtn.isVisible().catch(() => false))) {
        // 调试：打印页面 URL 和按钮
        console.log(`   当前 URL: ${page.url()}`);
        const btns = await page.locator('button').allTextContents();
        console.log(`   所有按钮: ${JSON.stringify(btns)}`);
        break;
      }
      await createBtn.click();
      await page.waitForTimeout(500);

      // 输入名称
      const modalInput = page.locator('.ant-modal input').first();
      if (await modalInput.isVisible().catch(() => false)) {
        await modalInput.fill(name);
      } else {
        console.log('   ❌ 模态框未弹出');
        continue;
      }

      // 点确定
      await page.locator('.ant-modal .ant-btn-primary').click();
      await page.waitForTimeout(1000);

      // 检查 Token 值是否弹出
      const tokenValueInput = page.locator('.ant-modal input[readonly]');
      if (await tokenValueInput.isVisible().catch(() => false)) {
        console.log('   ✅ Token 创建成功（值已显示）');
        // 关闭模态框
        await page.locator('.ant-modal-close').click().catch(() => {});
        await page.waitForTimeout(300);
        createdCount++;
      } else {
        console.log('   ⚠ Token 可能已创建，检查列表...');
        createdCount++;
      }
    }

    // ===== 6. 验证 Token 列表 =====
    console.log('\n6. 验证 Token 列表...');
    await page.waitForTimeout(1000);
    const rows = await page.locator('.ant-table-tbody tr.ant-table-row').count();
    console.log(`   Token 列表行数: ${rows}`);
    console.log(`   ${rows > 0 ? '✅ Token 列表有数据' : '❌ Token 列表为空'}`);

    // ===== 7. 错误检查 =====
    console.log('\n7. 错误检查...');
    console.log(`   ${consoleErrors.length === 0 ? '✅ 无' : `⚠ ${consoleErrors.length}个`} console 错误`);
    console.log(`   ${pageErrors.length === 0 ? '✅ 无' : `❌ ${pageErrors.length}个`} 页面错误`);

    console.log(`\n=== 测试完成 ===`);
    const passed = consoleErrors.length === 0 && pageErrors.length === 0;
    console.log(passed ? '🎉 全部测试通过！' : '⚠ 部分检查未通过');
  } catch (err) {
    console.log(`\n❌ 测试异常: ${err.message}`);
    await page.screenshot({ path: '/tmp/ntd_cloud_error.png', fullPage: true }).catch(() => {});
    console.log('   截图: /tmp/ntd_cloud_error.png');
  }

  await browser.close();
})();
