#!/usr/bin/env node
/**
 * 更新 HTML 模板中的 JavaScript 引用
 * 将原始 JS 文件路径替换为压缩后的版本
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');

const templatesDir = path.join(projectRoot, 'templates');
const distJsDir = path.join(projectRoot, 'static/dist/js');

// 获取所有压缩后的 JS 文件
function getMinifiedJsFiles() {
  const files = fs.readdirSync(distJsDir);
  return files.filter(file => file.endsWith('.min.js'));
}

// 更新 HTML 文件中的 JS 引用
function updateHtmlFile(filePath, minifiedFiles) {
  let content = fs.readFileSync(filePath, 'utf-8');
  let modified = false;

  minifiedFiles.forEach(minFile => {
    const originalFile = minFile.replace('.min.js', '.js');
    const minifiedPath = `/static/dist/js/${minFile}`;

    // 匹配多种 script 标签格式：
    // 1. <script src="/js/xxx.js" defer></script>
    // 2. <script src="/js/xxx.js?v=2" defer></script>
    // 3. <script type="module" src="/js/xxx.js"></script>
    const scriptRegex = new RegExp(
      `<script\\s+(?:type="module"\\s+)?src=["'](/js/${originalFile.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}(?:\\?v=\\d+)?)["']\\s*(?:defer)?>`,
      'g'
    );
    
    if (scriptRegex.test(content)) {
      content = content.replace(scriptRegex, `<script src="${minifiedPath}" defer>`);
      console.log(`  ✓ 更新: /js/${originalFile} -> ${minifiedPath}`);
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
  console.log('开始更新 HTML 模板中的 JavaScript 引用...\n');
  
  // 获取所有压缩后的 JS 文件
  const minifiedFiles = getMinifiedJsFiles();
  console.log(`找到 ${minifiedFiles.length} 个压缩后的 JS 文件\n`);
  
  // 查找所有 HTML 文件
  const htmlFiles = findHtmlFiles(templatesDir);
  console.log(`找到 ${htmlFiles.length} 个 HTML 文件\n`);
  
  let updatedCount = 0;
  
  htmlFiles.forEach(htmlFile => {
    const relativePath = path.relative(templatesDir, htmlFile);
    console.log(`处理: ${relativePath}`);
    
    if (updateHtmlFile(htmlFile, minifiedFiles)) {
      updatedCount++;
      console.log(`  ✓ 已更新\n`);
    } else {
      console.log(`  - 无需更新\n`);
    }
  });
  
  console.log(`\n更新完成！`);
  console.log(`共更新 ${updatedCount} 个 HTML 文件`);
}

main().catch(console.error);
