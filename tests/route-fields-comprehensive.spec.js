import { test, expect } from '@playwright/test';

/**
 * 动态路由字段重构全面测试套件
 * 
 * 测试范围：
 * 1. 所有 route_type 的路由创建和验证
 * 2. 所有 handler_type 的路由创建和验证
 * 3. 字段组合验证（完整覆盖文档规则）
 * 4. 边界条件测试
 * 5. CRUD完整流程测试
 * 6. 数据一致性测试
 */

test.describe('动态路由全面测试套件', () => {
  let authToken;
  let createdRouteIds = [];

  test.beforeEach(async ({ page }) => {
    // 直接访问动态路由管理页面（认证状态已通过storageState加载）
    await page.goto('http://localhost:8080/admin/dyn-routing');
    await page.waitForLoadState('networkidle');
    
    // 获取认证token
    authToken = await page.evaluate(() => localStorage.getItem('auth_token'));
    expect(authToken).not.toBeNull();
  });

  test.afterEach(async ({ page }) => {
    // 清理测试数据：删除创建的路由
    for (const routeId of createdRouteIds) {
      try {
        await page.evaluate(async (id) => {
          const response = await fetch(`/api/admin/dynamic-routes/${id}`, {
            method: 'DELETE',
            headers: {
              'Content-Type': 'application/json',
              'Authorization': `Bearer ${localStorage.getItem('auth_token')}`
            }
          });
          return response.ok;
        }, routeId);
        console.log(`已清理测试路由 ID: ${routeId}`);
      } catch (error) {
        console.error(`清理路由失败 ID: ${routeId}`, error.message);
      }
    }
  });

  // ==================== route_type 完整测试 ====================

  test.describe('Database类型路由测试', () => {
    test('创建database类型静态路由', async ({ page }) => {
      const routeId = await createRoute(page, {
        route_name: 'database-static-test',
        route_path: '/db-static-test',
        route_type: 'database',
        handler_type: 'static',
        inline_template: '<html><body>Database Static Content</body></html>',
        content_type_hint: 'text/html; charset=utf-8'
      });
      
      createdRouteIds.push(routeId);
      
      // 验证路由创建成功
      await verifyRouteInList(page, routeId, 'database');
      
      // 通过API验证数据
      const routeData = await fetchRouteData(page, routeId);
      expect(routeData.route_type).toBe('database');
      expect(routeData.inline_template).toContain('Database Static Content');
      expect(routeData.template_path).toBeNull();
    });

    test('创建database类型重定向路由', async ({ page }) => {
      const routeId = await createRoute(page, {
        route_name: 'database-redirect-test',
        route_path: '/db-redirect-test',
        route_type: 'database',
        handler_type: 'redirect',
        handler_config: JSON.stringify({
          target: '/new-location',
          status_code: 302,
          preserve_query: true
        })
      });
      
      createdRouteIds.push(routeId);
      
      // 验证路由创建成功
      await verifyRouteInList(page, routeId, 'database');
      
      const routeData = await fetchRouteData(page, routeId);
      expect(routeData.handler_type).toBe('redirect');
      const config = typeof routeData.handler_config === 'string' 
        ? JSON.parse(routeData.handler_config) 
        : routeData.handler_config;
      expect(config.target).toBe('/new-location');
      expect(config.status_code).toBe(302);
    });

    test('database类型禁止使用template_path', async ({ page }) => {
      await openAddModal(page);
      
      await page.fill('#routeName', 'validation-test');
      await page.fill('#routePath', '/validation-test');
      await page.selectOption('#routeType', 'database');
      await page.selectOption('#handlerType', 'static');
      await page.waitForTimeout(500);
      
      // 尝试设置template_path（通过JavaScript强制设置）
      await page.evaluate(() => {
        const input = document.getElementById('templatePath');
        if (input) input.value = 'templates/test.html';
      });
      
      await page.fill('#handlerConfig', '{}');
      await page.click('button:has-text("保存")');
      
      // 验证错误消息
      await page.waitForTimeout(2000);
      const errorVisible = await page.locator('#modalMessage').isVisible();
      expect(errorVisible).toBe(true);
      
      if (errorVisible) {
        const errorText = await page.locator('#modalMessage').textContent();
        expect(errorText).toContain('不支持 template_path');
      }
    });
  });

  test.describe('Memory类型路由测试', () => {
    test('创建memory类型路由', async ({ page }) => {
      const routeId = await createRoute(page, {
        route_name: 'memory-test',
        route_path: '/memory-test',
        route_type: 'memory',
        handler_type: 'static',
        inline_template: '<html><body>Memory Content</body></html>',
        content_type_hint: 'text/html; charset=utf-8'
      });
      
      createdRouteIds.push(routeId);
      
      await verifyRouteInList(page, routeId, 'memory');
      
      const routeData = await fetchRouteData(page, routeId);
      expect(routeData.route_type).toBe('memory');
      expect(routeData.inline_template).toContain('Memory Content');
      expect(routeData.template_path).toBeNull();
    });

    test('memory类型禁止使用template_path', async ({ page }) => {
      await openAddModal(page);
      
      await page.fill('#routeName', 'memory-validation-test');
      await page.fill('#routePath', '/memory-validation-test');
      await page.selectOption('#routeType', 'memory');
      await page.selectOption('#handlerType', 'static');
      await page.waitForTimeout(500);
      
      // 验证template_path字段不显示
      const templatePathVisible = await page.isVisible('#templatePathGroup');
      expect(templatePathVisible).toBe(false);
      
      // 验证inline_template字段显示
      const inlineTemplateVisible = await page.isVisible('#inlineTemplateGroup');
      expect(inlineTemplateVisible).toBe(true);
    });
  });

  test.describe('File类型路由测试', () => {
    test('创建file类型路由', async ({ page }) => {
      const routeId = await createRoute(page, {
        route_name: 'file-test',
        route_path: '/file-test',
        route_type: 'file',
        handler_type: 'static',
        template_path: 'templates/test.html',
        content_type_hint: 'text/html; charset=utf-8'
      });
      
      createdRouteIds.push(routeId);
      
      await verifyRouteInList(page, routeId, 'file');
      
      const routeData = await fetchRouteData(page, routeId);
      expect(routeData.route_type).toBe('file');
      expect(routeData.template_path).toBe('templates/test.html');
      expect(routeData.inline_template).toBeNull();
    });

    test('file类型必须提供template_path', async ({ page }) => {
      await openAddModal(page);
      
      await page.fill('#routeName', 'file-validation-test');
      await page.fill('#routePath', '/file-validation-test');
      await page.selectOption('#routeType', 'file');
      await page.selectOption('#handlerType', 'static');
      await page.waitForTimeout(500);
      
      // 不填写template_path
      await page.fill('#handlerConfig', '{}');
      await page.click('button:has-text("保存")');
      
      // 验证错误消息
      await page.waitForTimeout(2000);
      const errorVisible = await page.locator('#modalMessage').isVisible();
      expect(errorVisible).toBe(true);
      
      if (errorVisible) {
        const errorText = await page.locator('#modalMessage').textContent();
        expect(errorText).toContain('必须提供 template_path');
      }
    });

    test('file类型禁止使用inline_template', async ({ page }) => {
      await openAddModal(page);
      
      await page.fill('#routeName', 'file-inline-validation-test');
      await page.fill('#routePath', '/file-inline-validation-test');
      await page.selectOption('#routeType', 'file');
      await page.selectOption('#handlerType', 'static');
      await page.waitForTimeout(500);
      
      // 验证inline_template字段不显示
      const inlineTemplateVisible = await page.isVisible('#inlineTemplateGroup');
      expect(inlineTemplateVisible).toBe(false);
      
      // 验证template_path字段显示
      const templatePathVisible = await page.isVisible('#templatePathGroup');
      expect(templatePathVisible).toBe(true);
    });
  });

  // ==================== handler_type 完整测试 ====================

  test.describe('Handler类型测试', () => {
    test('static handler测试', async ({ page }) => {
      const routeId = await createRoute(page, {
        route_name: 'static-handler-test',
        route_path: '/static-handler',
        route_type: 'database',
        handler_type: 'static',
        inline_template: '<html><body>Static Handler Test</body></html>',
        content_type_hint: 'text/html; charset=utf-8'
      });
      
      createdRouteIds.push(routeId);
      
      const routeData = await fetchRouteData(page, routeId);
      expect(routeData.handler_type).toBe('static');
    });

    test('redirect handler测试', async ({ page }) => {
      const routeId = await createRoute(page, {
        route_name: 'redirect-handler-test',
        route_path: '/redirect-handler',
        route_type: 'database',
        handler_type: 'redirect',
        handler_config: JSON.stringify({
          target: '/target-path',
          status_code: 301,
          preserve_query: false
        })
      });
      
      createdRouteIds.push(routeId);
      
      const routeData = await fetchRouteData(page, routeId);
      expect(routeData.handler_type).toBe('redirect');
      const config = typeof routeData.handler_config === 'string' 
        ? JSON.parse(routeData.handler_config) 
        : routeData.handler_config;
      expect(config.target).toBe('/target-path');
    });

    test('proxy handler测试', async ({ page }) => {
      const routeId = await createRoute(page, {
        route_name: 'proxy-handler-test',
        route_path: '/proxy-handler',
        route_type: 'database',
        handler_type: 'proxy',
        handler_config: JSON.stringify({
          target: 'http://backend-service:8080',
          timeout: 5000,
          strip_prefix: false
        })
      });
      
      createdRouteIds.push(routeId);
      
      const routeData = await fetchRouteData(page, routeId);
      expect(routeData.handler_type).toBe('proxy');
    });

    test('custom handler测试', async ({ page }) => {
      const routeId = await createRoute(page, {
        route_name: 'custom-handler-test',
        route_path: '/custom-handler',
        route_type: 'database',
        handler_type: 'custom',
        handler_config: JSON.stringify({
          script: 'lua',
          source: 'function handle(req) return {status=200, body="OK"} end'
        })
      });
      
      createdRouteIds.push(routeId);
      
      const routeData = await fetchRouteData(page, routeId);
      expect(routeData.handler_type).toBe('custom');
    });

    test('template handler测试', async ({ page }) => {
      const routeId = await createRoute(page, {
        route_name: 'template-handler-test',
        route_path: '/template-handler',
        route_type: 'file',
        handler_type: 'template',
        template_path: 'templates/custom-page.html',
        handler_config: JSON.stringify({
          template_name: 'custom_page.html',
          context: {
            title: '自定义页面',
            content: '页面内容'
          }
        })
      });
      
      createdRouteIds.push(routeId);
      
      const routeData = await fetchRouteData(page, routeId);
      expect(routeData.handler_type).toBe('template');
    });
  });

  // ==================== 边界条件测试 ====================

  test.describe('边界条件测试', () => {
    test('空字段验证', async ({ page }) => {
      await openAddModal(page);
      
      // 不填写必填字段直接保存
      await page.click('button:has-text("保存")');
      
      // 验证模态框仍然打开（保存失败）
      await page.waitForTimeout(1000);
      const modalStillOpen = await page.isVisible('#routeModal.active');
      expect(modalStillOpen).toBe(true);
    });

    test('极长字段测试', async ({ page }) => {
      const longString = 'a'.repeat(10000);
      
      const routeId = await createRoute(page, {
        route_name: 'long-field-test',
        route_path: '/long-field-test',
        route_type: 'database',
        handler_type: 'static',
        inline_template: `<html><body>${longString}</body></html>`,
        content_type_hint: 'text/html; charset=utf-8'
      });
      
      createdRouteIds.push(routeId);
      
      const routeData = await fetchRouteData(page, routeId);
      expect(routeData.inline_template).toContain(longString);
    });

    test('特殊字符测试', async ({ page }) => {
      const specialChars = '<script>alert("XSS")</script>&"\'';
      
      const routeId = await createRoute(page, {
        route_name: 'special-chars-test',
        route_path: '/special-chars-test',
        route_type: 'database',
        handler_type: 'static',
        inline_template: `<html><body>${specialChars}</body></html>`,
        content_type_hint: 'text/html; charset=utf-8'
      });
      
      createdRouteIds.push(routeId);
      
      const routeData = await fetchRouteData(page, routeId);
      expect(routeData.inline_template).toContain(specialChars);
    });

    test('路径重复验证', async ({ page }) => {
      // 创建第一个路由
      const routeId1 = await createRoute(page, {
        route_name: 'duplicate-path-test-1',
        route_path: '/duplicate-path',
        route_type: 'database',
        handler_type: 'static',
        inline_template: '<html><body>First Route</body></html>'
      });
      createdRouteIds.push(routeId1);
      
      // 尝试创建相同路径的路由
      await openAddModal(page);
      await page.fill('#routeName', 'duplicate-path-test-2');
      await page.fill('#routePath', '/duplicate-path');
      await page.selectOption('#routeType', 'database');
      await page.selectOption('#handlerType', 'static');
      await page.fill('#inlineTemplate', '<html><body>Second Route</body></html>');
      await page.fill('#handlerConfig', '{}');
      await page.click('button:has-text("保存")');
      
      // 验证错误消息
      await page.waitForTimeout(2000);
      const errorVisible = await page.locator('#modalMessage').isVisible();
      expect(errorVisible).toBe(true);
    });
  });

  // ==================== CRUD完整流程测试 ====================

  test.describe('CRUD完整流程测试', () => {
    test('创建-读取-更新-删除完整流程', async ({ page }) => {
      // 1. 创建路由
      const routeId = await createRoute(page, {
        route_name: 'crud-full-test',
        route_path: '/crud-full-test',
        route_type: 'database',
        handler_type: 'static',
        inline_template: '<html><body>Initial Content</body></html>',
        content_type_hint: 'text/html; charset=utf-8'
      });
      createdRouteIds.push(routeId);
      
      // 2. 读取验证
      await verifyRouteInList(page, routeId, 'database');
      let routeData = await fetchRouteData(page, routeId);
      expect(routeData.inline_template).toContain('Initial Content');
      
      // 3. 更新路由
      await openEditModal(page, routeId);
      await page.fill('#routeName', 'crud-full-test-updated');
      await page.fill('#inlineTemplate', '<html><body>Updated Content</body></html>');
      await page.click('button:has-text("保存")');
      await page.waitForTimeout(2000);
      
      // 4. 验证更新
      routeData = await fetchRouteData(page, routeId);
      expect(routeData.route_name).toBe('crud-full-test-updated');
      expect(routeData.inline_template).toContain('Updated Content');
      
      // 5. 删除路由（在afterEach中统一清理）
      console.log('CRUD完整流程测试通过');
    });
  });

  test.describe('删除路由测试', () => {
    test('删除单个路由', async ({ page }) => {
      const routeId = await createRoute(page, {
        route_name: 'delete-test',
        route_path: '/delete-test',
        route_type: 'database',
        handler_type: 'static',
        inline_template: '<html><body>To be deleted</body></html>'
      });
      
      // 验证路由存在
      await verifyRouteInList(page, routeId, 'database');
      
      // 删除路由
      await deleteRoute(page, routeId);
      
      // 验证路由已删除
      await page.waitForTimeout(1000);
      await page.reload();
      await page.waitForLoadState('networkidle');
      
      const exists = await checkRouteExists(page, routeId);
      expect(exists).toBe(false);
    });

    test('批量删除路由', async ({ page }) => {
      const routeIds = [];
      
      // 创建多个路由
      for (let i = 0; i < 3; i++) {
        const routeId = await createRoute(page, {
          route_name: `batch-delete-test-${i}`,
          route_path: `/batch-delete-test-${i}`,
          route_type: 'database',
          handler_type: 'static',
          inline_template: `<html><body>Test ${i}</body></html>`
        });
        routeIds.push(routeId);
        createdRouteIds.push(routeId);
      }
      
      // 逐个删除
      for (const routeId of routeIds) {
        await deleteRoute(page, routeId);
      }
      
      // 验证所有路由已删除
      await page.reload();
      await page.waitForLoadState('networkidle');
      
      for (const routeId of routeIds) {
        const exists = await checkRouteExists(page, routeId);
        expect(exists).toBe(false);
      }
    });
  });

  // ==================== 数据一致性测试 ====================

  test.describe('数据一致性测试', () => {
    test('前后端数据一致性', async ({ page }) => {
      const testTemplate = '<html><body>Consistency Test</body></html>';
      
      const routeId = await createRoute(page, {
        route_name: 'consistency-test',
        route_path: '/consistency-test',
        route_type: 'database',
        handler_type: 'static',
        inline_template: testTemplate,
        content_type_hint: 'text/html; charset=utf-8'
      });
      createdRouteIds.push(routeId);
      
      // 从API获取数据
      const apiData = await fetchRouteData(page, routeId);
      
      // 从前端页面获取数据
      await openEditModal(page, routeId);
      const frontendTemplate = await page.inputValue('#inlineTemplate');
      
      // 验证一致性
      expect(apiData.inline_template).toBe(frontendTemplate);
      expect(apiData.inline_template).toBe(testTemplate);
      
      await page.click('.btn-secondary:has-text("取消")');
    });

    test('存储统计准确性', async ({ page }) => {
      // 获取初始统计
      await page.waitForLoadState('networkidle');
      const initialStats = await getStorageStats(page);
      console.log('初始统计:', initialStats);
      
      // 创建不同类型的路由
      const dbRouteId = await createRoute(page, {
        route_name: 'stats-db-test',
        route_path: '/stats-db-test',
        route_type: 'database',
        handler_type: 'static',
        inline_template: '<html><body>DB Route</body></html>'
      });
      createdRouteIds.push(dbRouteId);
      
      const memoryRouteId = await createRoute(page, {
        route_name: 'stats-memory-test',
        route_path: '/stats-memory-test',
        route_type: 'memory',
        handler_type: 'static',
        inline_template: '<html><body>Memory Route</body></html>'
      });
      createdRouteIds.push(memoryRouteId);
      
      const fileRouteId = await createRoute(page, {
        route_name: 'stats-file-test',
        route_path: '/stats-file-test',
        route_type: 'file',
        handler_type: 'static',
        template_path: 'templates/test.html'
      });
      createdRouteIds.push(fileRouteId);
      
      // 获取更新后的统计
      await page.reload();
      await page.waitForLoadState('networkidle');
      const updatedStats = await getStorageStats(page);
      console.log('更新后统计:', updatedStats);
      
      // 验证统计准确性：每个类型应该增加1，总数应该增加3
      const dbDiff = updatedStats.database - initialStats.database;
      const memoryDiff = updatedStats.memory - initialStats.memory;
      const fileDiff = updatedStats.file - initialStats.file;
      const totalDiff = updatedStats.total - initialStats.total;
      
      console.log('数据库路由增量:', dbDiff);
      console.log('内存路由增量:', memoryDiff);
      console.log('文件路由增量:', fileDiff);
      console.log('总数增量:', totalDiff);
      
      expect(dbDiff).toBe(1);
      expect(memoryDiff).toBe(1);
      expect(fileDiff).toBe(1);
      expect(totalDiff).toBe(3);
    });
  });

  // ==================== 辅助函数 ====================

  async function openAddModal(page) {
    await page.click('button:has-text("添加路由")');
    await page.waitForSelector('#routeModal.active');
  }

  async function openEditModal(page, routeId) {
    await page.evaluate((id) => {
      if (typeof editRoute === 'function') {
        editRoute(parseInt(id));
      }
    }, routeId);
    await page.waitForSelector('#routeModal.active');
  }

  async function createRoute(page, data) {
    await openAddModal(page);
    
    await page.fill('#routeName', data.route_name);
    await page.fill('#routePath', data.route_path);
    await page.selectOption('#routeType', data.route_type);
    await page.selectOption('#handlerType', data.handler_type);
    await page.waitForTimeout(500);
    
    if (data.inline_template !== undefined) {
      await page.fill('#inlineTemplate', data.inline_template);
    }
    
    if (data.template_path !== undefined) {
      await page.fill('#templatePath', data.template_path);
    }
    
    if (data.content_type_hint !== undefined) {
      await page.selectOption('#contentType', data.content_type_hint);
    }
    
    if (data.handler_config !== undefined) {
      await page.fill('#handlerConfig', data.handler_config);
    } else {
      await page.fill('#handlerConfig', '{}');
    }
    
    await page.click('button:has-text("保存")');
    await page.waitForTimeout(2000);
    
    // 验证成功消息
    const successMessage = await page.locator('.alert-success, .message-success, [class*="success"]').first();
    expect(await successMessage.isVisible()).toBe(true);
    
    // 关闭模态框并刷新页面（如果模态框还存在的话）
    const modalVisible = await page.isVisible('#routeModal.active');
    if (modalVisible) {
      await page.click('.btn-secondary:has-text("取消")');
      await page.waitForSelector('#routeModal', { state: 'hidden' });
    }
    await page.reload();
    await page.waitForLoadState('networkidle');
    await page.waitForSelector('#routesTableBody');
    
    // 获取创建的路由ID
    const routeId = await page.evaluate((routeName) => {
      const rows = document.querySelectorAll('#routesTableBody tr');
      for (const row of rows) {
        const cells = row.querySelectorAll('td');
        if (cells.length > 1 && cells[1].textContent.trim() === routeName) {
          return parseInt(cells[0].textContent.trim());
        }
      }
      return null;
    }, data.route_name);
    
    if (!routeId) {
      throw new Error(`无法找到创建的路由: ${data.route_name}`);
    }
    
    return routeId;
  }

  async function deleteRoute(page, routeId) {
    await page.evaluate(async (id) => {
      const response = await fetch(`/api/admin/dynamic-routes/${id}`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('auth_token')}`
        }
      });
      return response.ok;
    }, routeId);
  }

  async function verifyRouteInList(page, routeId, expectedType) {
    await page.reload();
    await page.waitForLoadState('networkidle');
    await page.waitForSelector('#routesTableBody');
    
    const exists = await checkRouteExists(page, routeId);
    expect(exists).toBe(true);
    
    // 验证存储类型
    const routeRow = page.locator(`#routesTableBody tr`).filter({
      has: page.locator('td').nth(0).filter({ hasText: new RegExp(`^${routeId}$`) })
    });
    
    if (expectedType) {
      const typeCell = await routeRow.locator('td:nth-child(7)').textContent();
      const typeMap = {
        'database': '数据库',
        'memory': '内存',
        'file': '文件'
      };
      expect(typeCell).toContain(typeMap[expectedType]);
    }
  }

  async function checkRouteExists(page, routeId) {
    return await page.evaluate((id) => {
      const rows = document.querySelectorAll('#routesTableBody tr');
      for (const row of rows) {
        if (row.textContent.includes(id.toString())) {
          return true;
        }
      }
      return false;
    }, routeId);
  }

  async function fetchRouteData(page, routeId) {
    return await page.evaluate(async (id) => {
      const response = await fetch(`/api/admin/dynamic-routes/${id}`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('auth_token')}`
        }
      });
      const result = await response.json();
      return result.success ? result.data : null;
    }, routeId);
  }

  async function getStorageStats(page) {
    return await page.evaluate(() => {
      return {
        total: parseInt(document.getElementById('totalRoutes').textContent),
        database: parseInt(document.getElementById('databaseRoutes').textContent),
        memory: parseInt(document.getElementById('memoryRoutes').textContent),
        file: parseInt(document.getElementById('fileRoutes').textContent)
      };
    });
  }
});