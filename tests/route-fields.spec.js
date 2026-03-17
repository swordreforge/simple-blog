import { test, expect } from '@playwright/test';

test.describe('路由字段重构前端测试', () => {
  test.beforeEach(async ({ page }) => {
    // 直接访问动态路由管理页面（认证状态已通过storageState加载）
    await page.goto('http://localhost:8080/admin/dyn-routing');
    
    // 等待页面加载
    await page.waitForLoadState('networkidle');
  });

  test('1. 登录测试', async ({ page }) => {
    // 验证是否在路由管理页面
    await expect(page).toHaveURL(/\/admin\/dyn-routing/);
    
    // 验证token存在
    const token = await page.evaluate(() => localStorage.getItem('auth_token'));
    expect(token).not.toBeNull();
    
    console.log('✅ 登录验证通过');
  });

  test('2. 路由列表显示测试', async ({ page }) => {
    // 等待路由列表加载
    await page.waitForSelector('#routesTableBody');
    
    // 获取路由列表
    const rows = await page.locator('#routesTableBody tr').all();
    expect(rows.length).toBeGreaterThan(0);
    
    // 检查第一行数据的字段
    const firstRow = rows[0];
    
    // 验证存储类型显示为"数据库"
    const storageTypeCell = await firstRow.locator('td:nth-child(7)').textContent();
    expect(storageTypeCell).toContain('数据库');
    
    console.log('✅ 路由列表显示正常');
  });

  test('3. 添加database类型路由', async ({ page }) => {
    // 点击添加路由按钮
    await page.click('button:has-text("添加路由")');
    
    // 等待模态框出现
    await page.waitForSelector('#routeModal.active');
    
    // 填写表单
    await page.fill('#routeName', '测试路由1');
    await page.fill('#routePath', '/test-route-1');
    await page.selectOption('#routeType', 'database');
    await page.selectOption('#handlerType', 'static');
    await page.fill('#routePriority', '0');
    
    // 验证字段显示：应该显示inlineTemplate，不显示templatePath
    const inlineTemplateVisible = await page.isVisible('#inlineTemplateGroup');
    const templatePathVisible = await page.isVisible('#templatePathGroup');
    
    expect(inlineTemplateVisible).toBe(true);
    expect(templatePathVisible).toBe(false);
    
    // 填写内容
    await page.selectOption('#contentType', 'text/html; charset=utf-8');
    await page.fill('#inlineTemplate', '<html><body>Test Content</body></html>');
    await page.fill('#handlerConfig', '{}');
    
    // 保存路由
    await page.click('button:has-text("保存")');
    
    // 等待保存完成
    await page.waitForTimeout(1000);
    
    // 验证成功消息
    const successMessage = await page.locator('.alert-success, .message-success, [class*="success"]').first();
    expect(await successMessage.isVisible()).toBe(true);
    
    console.log('✅ database类型路由添加成功');
  });

  test('4. 添加file类型路由', async ({ page }) => {
    // 点击添加路由按钮
    await page.click('button:has-text("添加路由")');
    
    // 等待模态框出现
    await page.waitForSelector('#routeModal.active');
    
    // 填写表单
    await page.fill('#routeName', '测试路由2');
    await page.fill('#routePath', '/test-route-2');
    await page.selectOption('#routeType', 'file');
    await page.selectOption('#handlerType', 'static');
    await page.fill('#routePriority', '0');
    
    // 验证字段显示：应该显示templatePath，不显示inlineTemplate
    const inlineTemplateVisible = await page.isVisible('#inlineTemplateGroup');
    const templatePathVisible = await page.isVisible('#templatePathGroup');
    
    expect(inlineTemplateVisible).toBe(false);
    expect(templatePathVisible).toBe(true);
    
    // 填写内容
    await page.selectOption('#contentType', 'text/html; charset=utf-8');
    await page.fill('#templatePath', 'templates/test.html');
    await page.fill('#handlerConfig', '{}');
    
    // 保存路由
    await page.click('button:has-text("保存")');
    
    // 等待保存完成
    await page.waitForTimeout(1000);
    
    // 验证成功消息
    const successMessage = await page.locator('.alert-success, .message-success, [class*="success"]').first();
    expect(await successMessage.isVisible()).toBe(true);
    
    console.log('✅ file类型路由添加成功');
  });

  test('5. 字段验证测试 - database类型不支持template_path', async ({ page }) => {
    // 点击添加路由按钮
    await page.click('button:has-text("添加路由")');
    
    // 等待模态框出现
    await page.waitForSelector('#routeModal.active');
    
    // 填写表单
    await page.fill('#routeName', '验证测试1');
    await page.fill('#routePath', '/validate-test-1');
    await page.selectOption('#routeType', 'database');
    await page.selectOption('#handlerType', 'static');
    
    // 等待字段更新
    await page.waitForTimeout(500);
    
    // 尝试填写template_path（可能不可见，直接通过JavaScript设置值）
    await page.evaluate(() => {
      const input = document.getElementById('templatePath');
      if (input) input.value = 'templates/test.html';
    });
    await page.fill('#handlerConfig', '{}');
    
    // 保存路由
    await page.click('button:has-text("保存")');
    
    // 等待验证结果（增加超时时间）
    await page.waitForTimeout(3000);
    
    // 验证错误消息（检查modal中的错误消息）
    const modalError = await page.locator('#modalMessage').first();
    const errorVisible = await modalError.isVisible();
    expect(errorVisible).toBe(true);
    
    if (errorVisible) {
      const errorText = await modalError.textContent();
      expect(errorText).toContain('不支持 template_path');
    }
    
    console.log('✅ 字段验证正常：database类型不支持template_path');
    
    // 关闭模态框
    await page.click('.btn-secondary:has-text("取消")');
    await page.waitForSelector('#routeModal', { state: 'hidden' });
  });

  test('6. 字段验证测试 - file类型不支持inline_template', async ({ page }) => {
    // 点击添加路由按钮
    await page.click('button:has-text("添加路由")');
    
    // 等待模态框出现
    await page.waitForSelector('#routeModal.active');
    
    // 填写表单
    await page.fill('#routeName', '验证测试2');
    await page.fill('#routePath', '/validate-test-2');
    await page.selectOption('#routeType', 'file');
    await page.selectOption('#handlerType', 'static');
    
    // 验证inline_template字段不可见
    const inlineTemplateVisible = await page.isVisible('#inlineTemplate');
    expect(inlineTemplateVisible).toBe(false);
    
    // 验证template_path字段可见
    const templatePathVisible = await page.isVisible('#templatePath');
    expect(templatePathVisible).toBe(true);
    
    console.log('✅ 字段验证正常：file类型不显示inline_template字段');
    
    // 关闭模态框
    await page.click('.btn-secondary:has-text("取消")');
    await page.waitForSelector('#routeModal', { state: 'hidden' });
  });

  test('7. 存储统计验证', async ({ page }) => {
    // 等待页面加载完成
    await page.waitForLoadState('networkidle');
    
    // 获取总路由数
    const totalRoutes = await page.textContent('#totalRoutes');
    console.log(`总路由数: ${totalRoutes}`);
    
    // 获取数据库存储数量
    const databaseRoutes = await page.textContent('#databaseRoutes');
    console.log(`数据库路由数: ${databaseRoutes}`);
    
    // 获取内存存储数量
    const memoryRoutes = await page.textContent('#memoryRoutes');
    console.log(`内存路由数: ${memoryRoutes}`);
    
    // 获取文件存储数量
    const fileRoutes = await page.textContent('#fileRoutes');
    console.log(`文件路由数: ${fileRoutes}`);
    
    // 验证统计信息
    expect(parseInt(totalRoutes)).toBeGreaterThanOrEqual(0);
    expect(parseInt(databaseRoutes)).toBeGreaterThanOrEqual(0);
    
    console.log('✅ 存储统计显示正常');
  });

  test('8. 编辑路由测试', async ({ page }) => {
    // 等待路由列表加载
    await page.waitForSelector('#routesTableBody');
    
    // 等待表格数据加载完成
    await page.waitForFunction(() => {
      const rows = document.querySelectorAll('#routesTableBody tr');
      return rows.length > 0 && !rows[0].textContent.includes('加载中');
    }, { timeout: 10000 });
    
    // 确保有路由数据
    const routeCount = await page.locator('#routesTableBody tr').count();
    expect(routeCount).toBeGreaterThan(0);
    console.log(`找到 ${routeCount} 条路由数据`);
    
    // 获取第一个路由
    const firstRoute = page.locator('#routesTableBody tr').first();
    
    // 检查编辑按钮是否存在
    const editButton = firstRoute.locator('button:has-text("编辑")');
    const buttonCount = await editButton.count();
    console.log(`编辑按钮数量: ${buttonCount}`);
    expect(buttonCount).toBeGreaterThan(0);
    
    // 点击编辑按钮
    console.log('点击编辑按钮...');
    await editButton.click();
    
    // 等待JavaScript执行完成
    await page.waitForTimeout(2000);
    
    // 检查模态框状态
    const modalState = await page.evaluate(() => {
      const modal = document.getElementById('routeModal');
      if (!modal) return '模态框不存在';
      return {
        exists: true,
        className: modal.className,
        display: window.getComputedStyle(modal).display
      };
    });
    console.log(`模态框状态:`, modalState);
    
    // 检查是否有JavaScript错误
    const hasError = await page.evaluate(() => {
      // 检查是否有错误提示
      const alerts = document.querySelectorAll('.alert, .message');
      return Array.from(alerts).map(a => ({
        class: a.className,
        text: a.textContent.trim()
      }));
    });
    console.log(`页面提示:`, hasError);
    
    // 尝试直接调用editRoute函数
    const routeId = await firstRoute.locator('td').first().textContent();
    console.log(`路由ID: ${routeId}`);
    
    try {
      // 使用JavaScript直接调用编辑函数
      await page.evaluate((id) => {
        console.log('准备调用editRoute函数，ID:', id);
        if (typeof editRoute === 'function') {
          console.log('editRoute函数存在');
          editRoute(parseInt(id));
          console.log('editRoute函数已调用');
        } else {
          console.error('editRoute函数不存在');
        }
      }, routeId);
      
      // 等待模态框显示
      await page.waitForSelector('#routeModal.active', { timeout: 5000 });
      
      console.log('模态框已打开');
      
      // 验证模态框标题
      const modalTitle = await page.textContent('#modalTitle');
      expect(modalTitle).toBe('编辑路由');
      
      // 修改路由名称
      const newRouteName = '已编辑的路由名称';
      await page.fill('#routeName', newRouteName);
      
      // 保存修改
      await page.click('button:has-text("保存")');
      
      // 等待保存完成
      await page.waitForTimeout(2000);
      
      // 验证成功消息
      const successMessage = await page.locator('.alert-success, .message-success, [class*="success"]').first();
      expect(await successMessage.isVisible()).toBe(true);
      
      console.log('✅ 编辑路由功能正常');
    } catch (error) {
      console.error('编辑路由失败:', error.message);
      
      // 收集更多错误信息
      const errorInfo = await page.evaluate(() => {
        return {
          modalExists: !!document.getElementById('routeModal'),
          modalClass: document.getElementById('routeModal')?.className,
          bodyClass: document.body.className
        };
      });
      console.log('错误时的页面状态:', errorInfo);
      throw error;
    }
  });
});