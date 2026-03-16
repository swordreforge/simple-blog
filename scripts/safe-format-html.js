#!/usr/bin/env node

/**
 * 安全的HTML格式化脚本
 * 读取压缩的HTML文件，格式化后保存到backups目录，不修改原文件
 */

import { readFileSync, writeFileSync, existsSync, statSync, mkdirSync } from 'fs';
import { resolve, join, dirname, basename } from 'path';
import { fileURLToPath } from 'url';
import prettier from 'prettier';

const __filename = fileURLToPath(import.meta.url);
const __dirname = join(__filename, '..');

// 获取项目根目录
const projectRoot = resolve(__dirname, '..');
const backupsDir = resolve(projectRoot, 'backups');

/**
 * 确保目录存在
 */
function ensureDirectoryExists(dirPath) {
  if (!existsSync(dirPath)) {
    mkdirSync(dirPath, { recursive: true });
  }
}

/**
 * 安全格式化单个HTML文件
 * @param {string} sourcePath - 源文件路径
 * @param {string} outputPath - 输出文件路径
 * @returns {Promise<boolean>} - 是否成功格式化
 */
async function safeFormatFile(sourcePath, outputPath) {
  try {
    const absoluteSourcePath = resolve(sourcePath);
    
    // 检查源文件是否存在
    if (!existsSync(absoluteSourcePath)) {
      console.error(`❌ 源文件不存在: ${absoluteSourcePath}`);
      return false;
    }

    // 检查是否为文件
    const stats = statSync(absoluteSourcePath);
    if (!stats.isFile()) {
      console.error(`❌ 不是文件: ${absoluteSourcePath}`);
      return false;
    }

    console.log(`📄 正在格式化: ${basename(absoluteSourcePath)}`);

    // 读取文件内容
    const content = readFileSync(absoluteSourcePath, 'utf-8');

    // 使用 Prettier 格式化
    const formatted = await prettier.format(content, {
      filepath: absoluteSourcePath,
      parser: 'html',
    });

    // 确保输出目录存在
    const outputDir = dirname(outputPath);
    ensureDirectoryExists(outputDir);

    // 写入格式化后的文件
    writeFileSync(outputPath, formatted, 'utf-8');
    console.log(`✅ 已保存到: ${outputPath}`);
    return true;
  } catch (error) {
    console.error(`❌ 格式化失败 ${sourcePath}:`, error.message);
    return false;
  }
}

/**
 * 主函数
 */
async function main() {
  console.log('🎨 安全HTML格式化工具');
  console.log('====================\n');

  // 确保backups目录存在
  ensureDirectoryExists(backupsDir);

  // 定义要格式化的文件列表
  const filesToFormat = [
    'templates/index.html',
    'templates/passage.html',
    'templates/about.html',
    'templates/collect.html',
    'templates/friends.html',
    'templates/markdown-editor.html',
    'templates/admin/admin.html',
    'templates/admin/filemanager.html',
    'templates/status/302.html',
    'templates/status/401.html',
    'templates/status/404.html',
    'templates/status/405.html',
    'templates/status/409.html',
    'templates/status/423.html',
    'templates/status/500.html',
    'templates/status/999.html'
  ];

  let successCount = 0;
  let failCount = 0;

  for (const relativePath of filesToFormat) {
    const sourcePath = resolve(projectRoot, relativePath);
    const outputPath = resolve(backupsDir, relativePath + '.formatted');

    const success = await safeFormatFile(sourcePath, outputPath);
    if (success) {
      successCount++;
    } else {
      failCount++;
    }
  }

  console.log('\n====================');
  console.log(`✅ 完成! 成功格式化 ${successCount} 个文件`);
  if (failCount > 0) {
    console.log(`❌ 失败 ${failCount} 个文件`);
  }
  console.log(`📁 所有格式化文件保存在: ${backupsDir}`);
}

// 运行主函数
main().catch((error) => {
  console.error('❌ 未捕获的错误:', error);
  process.exit(1);
});