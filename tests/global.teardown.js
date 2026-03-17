/**
 * Playwright 全局清理脚本
 * 在所有测试运行后执行
 */

async function globalTeardown(config) {
  console.log('🧹 全局清理完成');
}

module.exports = globalTeardown;