#!/usr/bin/env node

/**
 * HTML 格式化脚本
 * 支持对特定的 HTML 文件或目录进行格式化
 * 使用 Prettier 进行格式化，遵循项目的 .prettierrc 配置
 */

import { readFileSync, writeFileSync, existsSync, statSync } from 'fs';
import { resolve, join } from 'path';
import { fileURLToPath } from 'url';
import prettier from 'prettier';

const __filename = fileURLToPath(import.meta.url);
const __dirname = join(__filename, '..');

// 获取项目根目录
const projectRoot = resolve(__dirname, '..');

/**
 * 格式化单个 HTML 文件
 * @param {string} filePath - 文件路径
 * @returns {Promise<boolean>} - 是否成功格式化
 */
async function formatFile(filePath) {
  try {
    const absolutePath = resolve(filePath);
    
    // 检查文件是否存在
    if (!existsSync(absolutePath)) {
      console.error(`❌ 文件不存在: ${absolutePath}`);
      return false;
    }

    // 检查是否为文件
    const stats = statSync(absolutePath);
    if (!stats.isFile()) {
      console.error(`❌ 不是文件: ${absolutePath}`);
      return false;
    }

    console.log(`📄 正在格式化: ${absolutePath}`);

    // 读取文件内容
    const content = readFileSync(absolutePath, 'utf-8');

    // 使用 Prettier 格式化
    const formatted = await prettier.format(content, {
      filepath: absolutePath,
      parser: 'html',
    });

    // 写回文件
    writeFileSync(absolutePath, formatted, 'utf-8');
    console.log(`✅ 格式化完成: ${absolutePath}`);
    return true;
  } catch (error) {
    console.error(`❌ 格式化失败 ${filePath}:`, error.message);
    return false;
  }
}

/**
 * 格式化目录中的所有 HTML 文件
 * @param {string} dirPath - 目录路径
 * @param {boolean} recursive - 是否递归处理子目录
 * @returns {Promise<number>} - 成功格式化的文件数量
 */
async function formatDirectory(dirPath, recursive = false) {
  const { readdirSync, statSync } = await import('fs');
  const { resolve, join } = await import('path');
  
  const absolutePath = resolve(dirPath);
  
  if (!existsSync(absolutePath)) {
    console.error(`❌ 目录不存在: ${absolutePath}`);
    return 0;
  }

  const stats = statSync(absolutePath);
  if (!stats.isDirectory()) {
    console.error(`❌ 不是目录: ${absolutePath}`);
    return 0;
  }

  console.log(`📁 正在处理目录: ${absolutePath}`);

  let successCount = 0;
  const entries = readdirSync(absolutePath);

  for (const entry of entries) {
    const entryPath = join(absolutePath, entry);
    const entryStats = statSync(entryPath);

    if (entryStats.isDirectory() && recursive) {
      // 递归处理子目录
      successCount += await formatDirectory(entryPath, true);
    } else if (entryStats.isFile() && entry.endsWith('.html')) {
      // 处理 HTML 文件
      const success = await formatFile(entryPath);
      if (success) successCount++;
    }
  }

  return successCount;
}

/**
 * 显示帮助信息
 */
function showHelp() {
  console.log(`
HTML 格式化脚本
===============

用法:
  node scripts/format-html.js <path> [选项]

参数:
  path              要格式化的文件或目录路径（相对于项目根目录或绝对路径）

选项:
  -r, --recursive   递归处理子目录（仅对目录有效）
  -h, --help        显示帮助信息

示例:
  # 格式化单个文件
  node scripts/format-html.js templates/index.html

  # 格式化目录中的所有 HTML 文件（不递归）
  node scripts/format-html.js templates

  # 递归格式化目录及其子目录中的所有 HTML 文件
  node scripts/format-html.js templates -r

  # 使用绝对路径格式化
  node scripts/format-html.js /home/user/project/templates/index.html

配置:
  使用项目的 .prettierrc 配置文件进行格式化
  当前配置:
    - 单引号: true
    - 分号: true
    - 缩进: 2 空格
    - 行宽: 100 字符
  `);
}

/**
 * 主函数
 */
async function main() {
  const args = process.argv.slice(2);

  // 显示帮助
  if (args.includes('-h') || args.includes('--help') || args.length === 0) {
    showHelp();
    process.exit(0);
  }

  // 解析参数
  let targetPath = args[0];
  const recursive = args.includes('-r') || args.includes('--recursive');

  // 如果是相对路径，解析为绝对路径
  if (!targetPath.startsWith('/')) {
    targetPath = resolve(projectRoot, targetPath);
  }

  console.log('🎨 HTML 格式化工具');
  console.log('==================\n');

  // 判断是文件还是目录
  let successCount = 0;
  try {
    const stats = statSync(targetPath);
    
    if (stats.isFile()) {
      const success = await formatFile(targetPath);
      if (success) successCount = 1;
    } else if (stats.isDirectory()) {
      successCount = await formatDirectory(targetPath, recursive);
    }
  } catch (error) {
    console.error(`❌ 错误: ${error.message}`);
    process.exit(1);
  }

  console.log('\n==================');
  console.log(`✅ 完成! 成功格式化 ${successCount} 个文件`);
}

// 运行主函数
main().catch((error) => {
  console.error('❌ 未捕获的错误:', error);
  process.exit(1);
});
