import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 60 * 1000, // 60秒超时
  retries: 1, // 失败重试1次
  
  // 全局设置：使用保存的认证状态
  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on-first-retry', // 第一次重试时记录trace
    screenshot: 'only-on-failure', // 失败时截图
    video: 'retain-on-failure', // 失败时保留视频
  },
  
  // 项目配置：为需要认证的测试使用已保存的状态
  projects: [
    {
      name: 'authenticated',
      use: {
        ...devices['Desktop Chrome'],
        // 使用已保存的认证状态
        storageState: 'auth.json',
      },
      testMatch: /route-fields.*\.spec\.js$/, // 匹配路由字段测试文件
    },
    {
      name: 'default',
      use: {
        ...devices['Desktop Chrome'],
      },
      testIgnore: /route-fields.*\.spec\.js$/, // 排除路由字段测试文件
    },
  ],
  
  // 测试运行前执行的设置
  globalSetup: './tests/global.setup.js',
  
  // 测试运行后执行的清理
  globalTeardown: './tests/global.teardown.js',
});