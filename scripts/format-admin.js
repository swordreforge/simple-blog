#!/usr/bin/env node

/**
 * 简化的HTML格式化脚本
 * 对admin.html进行基础格式化处理
 */

import { readFileSync, writeFileSync, existsSync, statSync, mkdirSync } from 'fs';
import { resolve, join, dirname, basename } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = join(__filename, '..');
const projectRoot = resolve(__dirname, '..');
const backupsDir = resolve(projectRoot, 'backups');

/**
 * 基础HTML格式化
 * 只做最基本的换行和缩进处理，不破坏现有结构
 */
function basicFormatHTML(html) {
  // 在HTML标签后添加换行
  let formatted = html
    .replace(/></g, '>\n<')
    .replace(/<([a-zA-Z][^>]*)>/g, (match, tag) => {
      // 在开始标签后换行（除了自闭合标签）
      if (tag.endsWith('/')) {
        return match;
      }
      if (['br', 'hr', 'img', 'input', 'meta', 'link'].some(t => tag.startsWith(t))) {
        return match;
      }
      return match + '\n';
    });

  // 在</p>、</div>、</li>等块级元素后添加空行
  formatted = formatted.replace(/<\/(p|div|li|ul|ol|h[1-6]|section|article|aside|header|footer|nav)>/g, '$&\n');

  // 基本缩进处理
  const lines = formatted.split('\n');
  let indentLevel = 0;
  const indentedLines = lines.map(line => {
    const trimmed = line.trim();
    if (!trimmed) return '';

    // 减少缩进
    if (trimmed.startsWith('</')) {
      indentLevel = Math.max(0, indentLevel - 1);
    }

    const result = '  '.repeat(indentLevel) + trimmed;

    // 增加缩进
    if (trimmed.startsWith('<') && !trimmed.startsWith('</') && !trimmed.startsWith('<!')) {
      const tagName = trimmed.match(/<(\w+)/);
      if (tagName && !['br', 'hr', 'img', 'input', 'meta', 'link'].includes(tagName[1])) {
        if (!trimmed.endsWith('/>')) {
          indentLevel++;
        }
      }
    }

    return result;
  });

  return indentedLines.filter(line => line.trim() !== '').join('\n');
}

/**
 * 格式化admin.html文件
 */
function formatAdminHTML() {
  try {
    const sourcePath = resolve(projectRoot, 'templates/admin/admin.html');
    const outputPath = resolve(backupsDir, 'templates/admin/admin.html.formatted');

    // 检查源文件是否存在
    if (!existsSync(sourcePath)) {
      console.error(`❌ 源文件不存在: ${sourcePath}`);
      return false;
    }

    console.log(`📄 正在格式化: admin.html`);

    // 读取文件内容
    const content = readFileSync(sourcePath, 'utf-8');

    // 使用基础格式化
    const formatted = basicFormatHTML(content);

    // 确保输出目录存在
    const outputDir = dirname(outputPath);
    if (!existsSync(outputDir)) {
      mkdirSync(outputDir, { recursive: true });
    }

    // 写入格式化后的文件
    writeFileSync(outputPath, formatted, 'utf-8');
    console.log(`✅ 已保存到: ${outputPath}`);
    return true;
  } catch (error) {
    console.error(`❌ 格式化失败 admin.html:`, error.message);
    return false;
  }
}

// 运行格式化
formatAdminHTML();