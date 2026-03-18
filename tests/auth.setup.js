/**
 * Playwright 认证设置脚本
 * 用于保存登录状态到 auth.json 文件，供其他测试复用
 */

import { test, expect } from '@playwright/test';

test('保存登录状态', async ({ page }) => {
  console.log('开始登录流程...');
  
  // 访问首页
  await page.goto('http://localhost:8080/');
  await page.waitForLoadState('networkidle');
  
  // 检查是否需要登录
  const loginBtn = page.locator('#loginBtn');
  if (await loginBtn.count() === 0) {
    console.log('已经登录，跳过登录步骤');
    return;
  }
  
  // 按 'l' 键打开登录模态框
  await page.keyboard.press('l');
  await page.waitForSelector('#loginModal.active', { timeout: 5000 });
  console.log('登录模态框已打开');
  
  // 填写登录信息
  await page.fill('#loginUsername', 'admin');
  await page.fill('#loginPassword', 'admin123');
  
  // 点击登录按钮
  await page.click('#loginSubmitBtn');
  
  // 等待登录完成
  await page.waitForSelector('#loginModal', { state: 'hidden', timeout: 5000 });
  
  // 等待token保存到localStorage
  await page.waitForFunction(() => {
    return localStorage.getItem('auth_token') !== null;
  }, { timeout: 5000 });
  
  console.log('登录成功，保存认证状态...');
  
  // 保存当前上下文的状态到文件
  await page.context().storageState({ path: 'auth.json' });
  
  console.log('✅ 认证状态已保存到 auth.json');
});