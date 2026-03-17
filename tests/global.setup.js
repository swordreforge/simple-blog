/**
 * Playwright 全局设置脚本
 * 在所有测试运行前执行，确保认证状态存在
 */

const { chromium } = require('@playwright');
const path = require('path');
const fs = require('fs');

async function globalSetup(config) {
  console.log('🚀 开始全局设置...');
  
  const authFile = path.join(__dirname, '..', 'auth.json');
  
  // 检查认证文件是否已存在
  if (fs.existsSync(authFile)) {
    console.log('✅ 认证文件已存在，跳过登录');
    return;
  }
  
  console.log('📝 认证文件不存在，开始登录...');
  
  // 启动浏览器
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();
  
  try {
    // 访问首页
    await page.goto('http://localhost:8080/');
    await page.waitForLoadState('networkidle');
    
    // 检查是否需要登录
    const loginBtn = page.locator('#loginBtn');
    if (await loginBtn.count() > 0) {
      // 按 'l' 键打开登录模态框
      await page.keyboard.press('l');
      await page.waitForSelector('#loginModal.active', { timeout: 5000 });
      
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
      
      console.log('✅ 登录成功');
    }
    
    // 保存认证状态
    await context.storageState({ path: authFile });
    console.log('✅ 认证状态已保存到 auth.json');
    
  } catch (error) {
    console.error('❌ 登录失败:', error.message);
    throw error;
  } finally {
    await browser.close();
  }
}

module.exports = globalSetup;