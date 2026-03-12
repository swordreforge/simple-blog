#!/usr/bin/env node
/**
 * 恢复 HTML 模板中的 JavaScript 引用
 * 将压缩后的文件路径恢复为原始路径
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');

const templatesDir = path.join(projectRoot, 'templates');

// 获取所有已压缩的 JS 文件
function getCompressedJsFiles() {
  const jsDir = path.join(projectRoot, 'templates/js');
  if (!fs.existsSync(jsDir)) {
    return [];
  }
  const files = fs.readdirSync(jsDir);
  return files.filter(file => file.endsWith('.js') && file !== 'highlight.min.js');
}

// 恢复 HTML 文件中的 JS 引用
function restoreHtmlFile(filePath, jsFiles) {
  let content = fs.readFileSync(filePath, 'utf-8');
  let modified = false;

  jsFiles.forEach(jsFile => {
    const compressedPath = `/static/dist/js/${jsFile.replace('.js', '.min.js')}`;
    const originalPath = `/js/${jsFile}`;

    // 匹配压缩后的路径
    const scriptRegex = new RegExp(
      `<script\\s+(?:type="module"\\s+)?src=["']${compressedPath.replace(/[.*+?^${}()|[\\]\\]/g, '\\$&')}["']\\s*(?:defer)?>`,
      'g'
    );
    
    if (scriptRegex.test(content)) {
      content = content.replace(scriptRegex, `<script src="${originalPath}" defer>`);
      console.log(`  ✓ 恢复: ${compressedPath} -> ${originalPath}`);
      modified = true;
    }
  });

  if (modified) {
    fs.writeFileSync(filePath, content, 'utf-8');
    return true;
  }
  return false;
}

// 递归查找所有 HTML 文件
function findHtmlFiles(dir, fileList = []) {
  const files = fs.readdirSync(dir);
  
  files.forEach(file => {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    
    if (stat.isDirectory()) {
      findHtmlFiles(filePath, fileList);
    } else if (file.endsWith('.html')) {
      fileList.push(filePath);
    }
  });
  
  return fileList;
}

async function main() {
  console.log('开始恢复 HTML 模板中的 JavaScript 引用...\n');
  
  // 获取所有 JS 文件
  const jsFiles = getCompressedJsFiles();
  console.log(`找到 ${jsFiles.length} 个 JS 文件\n`);
  
  // 查找所有 HTML 文件
  const htmlFiles = findHtmlFiles(templatesDir);
  console.log(`找到 ${htmlFiles.length} 个 HTML 文件\n`);
  
  let restoredCount = 0;
  
  htmlFiles.forEach(htmlFile => {
    const relativePath = path.relative(templatesDir, htmlFile);
    console.log(`处理: ${relativePath}`);
    
    if (restoreHtmlFile(htmlFile, jsFiles)) {
      restoredCount++;
      console.log(`  ✓ 已恢复\n`);
    } else {
      console.log(`  - 无需恢复\n`);
    }
  });
  
  console.log(`\n恢复完成！`);
  console.log(`共恢复 ${restoredCount} 个 HTML 文件`);
  console.log(`\n所有 JS 文件现在指向原始路径，但文件内容已经是压缩版本`);
}

main().catch(console.error);