import { test, expect } from '@playwright/test';

/**
 * 文件上传功能测试
 * 
 * 测试流程：
 * 1. 用户访问主界面/
 * 2. 按 l 按键开启登录获取 token
 * 3. 访问 @/admin/dyn-routing
 * 4. 点击添加路由
 * 5. 在弹出的模态框中填写路由名称、路由路径、处理器选择静态内容
 * 6. 点击上传文件
 * 7. 传入当前路径下的 test.html
 * 8. 查看 <textarea id="inlineTemplate"> 这个里面是否填充了我们上传的文件并显示内容
 * 9. 如果是返回成功否则失败
 */

test.describe('文件上传功能测试', () => {
  let authToken;
  const testFilePath = '/home/swordreforge/projects/rustblog-new/rustblog/test.html';
  const expectedContent = `<!DOCTYPE html><html lang="zh-CN"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>测试页面</title></head><body><h1>这是一个测试页面2</h1><p>用于测试 file 类型的动态路由</p></body></html>`;

  test.beforeEach(async ({ page }) => {
    // 访问首页
    await page.goto('http://localhost:8080/');
    await page.waitForLoadState('networkidle');
    
    // 按 'l' 键打开登录模态框
    await page.keyboard.press('l');
    
    // 等待登录模态框显示（使用更灵活的选择器）
    await page.waitForFunction(() => {
      const modal = document.getElementById('loginModal');
      return modal && modal.classList.contains('active');
    }, { timeout: 5000 });
    
    // 填写登录信息
    await page.fill('#loginUsername', 'admin');
    await page.fill('#loginPassword', 'admin123');
    
    // 点击登录按钮
    await page.click('#loginSubmitBtn');
    
    // 等待登录完成
    await page.waitForFunction(() => {
      const modal = document.getElementById('loginModal');
      return modal && !modal.classList.contains('active');
    }, { timeout: 5000 });
    
    // 等待token保存到localStorage
    await page.waitForFunction(() => {
      return localStorage.getItem('auth_token') !== null;
    }, { timeout: 5000 });
    
    // 获取认证token
    authToken = await page.evaluate(() => localStorage.getItem('auth_token'));
    expect(authToken).not.toBeNull();
    
    // 访问动态路由管理页面
    await page.goto('http://localhost:8080/admin/dyn-routing');
    await page.waitForLoadState('networkidle');
  });

  test('上传文件到内联模板字段', async ({ page }) => {
    console.log('步骤1: 点击添加路由按钮');
    await page.click('button:has-text("添加路由")');
    await page.waitForSelector('#routeModal.active');
    
    console.log('步骤2: 填写路由名称');
    await page.fill('#routeName', 'file-upload-test');
    
    console.log('步骤3: 填写路由路径');
    await page.fill('#routePath', '/file-upload-test');
    
    console.log('步骤4: 选择路由类型为 database');
    await page.selectOption('#routeType', 'database');
    
    console.log('步骤5: 选择处理器类型为静态内容');
    await page.selectOption('#handlerType', 'static');
    
    console.log('步骤6: 等待内联模板字段显示');
    await page.waitForSelector('#inlineTemplateGroup', { state: 'visible' });
    await page.waitForSelector('#uploadFileBtn', { state: 'visible' });
    
    console.log('步骤7: 上传文件');
    const fileInput = page.locator('#fileInput');
    
    // 确保文件输入元素可见
    await fileInput.evaluate(el => el.style.display = 'block');
    await fileInput.setInputFiles(testFilePath);
    
    console.log('步骤8: 等待文件内容填充到内联模板字段');
    // 等待文件读取完成，通过检查内联模板字段是否有内容
    await page.waitForFunction(() => {
      const textarea = document.getElementById('inlineTemplate');
      return textarea && textarea.value.length > 0;
    }, { timeout: 10000 });
    
    console.log('步骤9: 获取内联模板字段的内容');
    const actualContent = await page.inputValue('#inlineTemplate');
    
    console.log('步骤10: 验证内容是否正确');
    console.log('预期内容:', expectedContent);
    console.log('实际内容:', actualContent);
    
    expect(actualContent).toBe(expectedContent);
    
    console.log('✅ 测试成功：文件内容已正确填充到内联模板字段');
    
    // 清理：关闭模态框
    await page.click('.btn-secondary:has-text("取消")');
    await page.waitForSelector('#routeModal', { state: 'hidden' });
  });

  test('验证上传文件后字段自动填充', async ({ page }) => {
    console.log('步骤1: 点击添加路由按钮');
    await page.click('button:has-text("添加路由")');
    await page.waitForSelector('#routeModal.active');
    
    console.log('步骤2: 填写路由基本信息');
    await page.fill('#routeName', 'file-upload-auto-test');
    await page.fill('#routePath', '/file-upload-auto-test');
    await page.selectOption('#routeType', 'database');
    await page.selectOption('#handlerType', 'static');
    
    console.log('步骤3: 上传文件');
    const fileInput = page.locator('#fileInput');
    await fileInput.evaluate(el => el.style.display = 'block');
    await fileInput.setInputFiles(testFilePath);
    
    console.log('步骤4: 等待文件读取完成');
    await page.waitForFunction(() => {
      const textarea = document.getElementById('inlineTemplate');
      return textarea && textarea.value.length > 0;
    }, { timeout: 10000 });
    
    console.log('步骤5: 验证处理器类型是否自动设置为 static');
    const handlerType = await page.inputValue('#handlerType');
    expect(handlerType).toBe('static');
    
    console.log('步骤6: 验证 Content-Type 是否自动设置');
    const contentType = await page.inputValue('#contentType');
    expect(contentType).toBe('text/html; charset=utf-8');
    
    console.log('步骤7: 验证处理器配置是否自动生成');
    const handlerConfig = await page.inputValue('#handlerConfig');
    const config = JSON.parse(handlerConfig);
    expect(config.type).toBe('static');
    expect(config.headers).toBeDefined();
    expect(config.headers['Cache-Control']).toBe('public, max-age=3600');
    
    console.log('步骤8: 验证内联模板内容');
    const actualContent = await page.inputValue('#inlineTemplate');
    expect(actualContent).toBe(expectedContent);
    
    console.log('✅ 测试成功：上传文件后所有相关字段都正确自动填充');
    
    // 清理：关闭模态框
    await page.click('.btn-secondary:has-text("取消")');
    await page.waitForSelector('#routeModal', { state: 'hidden' });
  });

  test('验证文件大小限制', async ({ page }) => {
    console.log('步骤1: 点击添加路由按钮');
    await page.click('button:has-text("添加路由")');
    await page.waitForSelector('#routeModal.active');
    
    console.log('步骤2: 填写基本信息');
    await page.fill('#routeName', 'file-upload-size-test');
    await page.fill('#routePath', '/file-upload-size-test');
    await page.selectOption('#routeType', 'database');
    await page.selectOption('#handlerType', 'static');
    
    console.log('步骤3: 创建一个超过1MB的文件');
    const largeContent = '<html><body>' + 'x'.repeat(2000000) + '</body></html>';
    const largeFile = await page.evaluateHandle(async (content) => {
      const blob = new Blob([content], { type: 'text/html' });
      return new File([blob], 'large.html', { type: 'text/html' });
    }, largeContent);
    
    console.log('步骤4: 尝试上传大文件');
    const fileInput = page.locator('#fileInput');
    await fileInput.evaluate(el => el.style.display = 'block');
    await fileInput.setInputFiles(largeFile);
    
    console.log('步骤5: 等待可能的错误提示');
    await page.waitForTimeout(2000);
    
    console.log('步骤6: 验证内联模板内容未改变（文件被拒绝）');
    const actualContent = await page.inputValue('#inlineTemplate');
    expect(actualContent).toBe('');
    
    console.log('✅ 测试成功：超过大小限制的文件被正确拒绝');
    
    // 清理：关闭模态框
    await page.click('.btn-secondary:has-text("取消")');
    await page.waitForSelector('#routeModal', { state: 'hidden' });
  });
});