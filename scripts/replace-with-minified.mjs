#!/usr/bin/env node
/**
 * 将压缩后的 JS 文件替换原始文件
 * 保留原始文件的备份
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');

const templatesJsDir = path.join(projectRoot, 'templates/js');
const distJsDir = path.join(projectRoot, 'static/dist/js');
const backupDir = path.join(projectRoot, 'backups/original-js');

// 创建备份目录
if (!fs.existsSync(backupDir)) {
  fs.mkdirSync(backupDir, { recursive: true });
  console.log(`创建备份目录: ${backupDir}`);
}

// 获取所有压缩后的 JS 文件
function getMinifiedJsFiles() {
  if (!fs.existsSync(distJsDir)) {
    return [];
  }
  const files = fs.readdirSync(distJsDir);
  return files.filter(file => file.endsWith('.min.js'));
}

// 备份原始文件
function backupOriginalFile(originalFile) {
  const backupFile = path.join(backupDir, path.basename(originalFile));
  fs.copyFileSync(originalFile, backupFile);
  console.log(`  ✓ 备份: ${path.basename(originalFile)} -> backups/original-js/`);
}

// 替换文件
function replaceWithMinified() {
  console.log('开始替换 JS 文件...\n');
  
  const minifiedFiles = getMinifiedJsFiles();
  console.log(`找到 ${minifiedFiles.length} 个压缩后的 JS 文件\n`);
  
  if (minifiedFiles.length === 0) {
    console.log('没有找到压缩文件，请先运行压缩脚本');
    return;
  }
  
  let replacedCount = 0;
  let skippedCount = 0;
  
  minifiedFiles.forEach(minFile => {
    const originalFile = minFile.replace('.min.js', '.js');
    const originalPath = path.join(templatesJsDir, originalFile);
    const minifiedPath = path.join(distJsDir, minFile);
    
    // 检查原始文件是否存在
    if (!fs.existsSync(originalPath)) {
      console.log(`  ✗ 跳过: ${originalFile} (原始文件不存在)`);
      skippedCount++;
      return;
    }
    
    // 检查原始文件是否已经是压缩版本
    const originalContent = fs.readFileSync(originalPath, 'utf-8');
    if (originalContent.includes('License: MIT') && originalContent.includes('Terser')) {
      console.log(`  - 跳过: ${originalFile} (已经是压缩版本)`);
      skippedCount++;
      return;
    }
    
    try {
      // 备份原始文件
      backupOriginalFile(originalPath);
      
      // 读取压缩文件内容（添加压缩标记）
      let minifiedContent = fs.readFileSync(minifiedPath, 'utf-8');
      const comment = `/* Terser compressed file */\n`;
      if (!minifiedContent.startsWith('/*')) {
        minifiedContent = comment + minifiedContent;
      }
      
      // 写入原始文件位置
      fs.writeFileSync(originalPath, minifiedContent, 'utf-8');
      
      const originalSize = fs.statSync(backupDir + '/' + path.basename(originalFile)).size;
      const compressedSize = fs.statSync(originalPath).size;
      const reduction = ((1 - compressedSize / originalSize) * 100).toFixed(1);
      
      console.log(`  ✓ 替换: ${originalFile} (压缩率: ${reduction}%)`);
      replacedCount++;
    } catch (error) {
      console.log(`  ✗ 失败: ${originalFile} - ${error.message}`);
      skippedCount++;
    }
  });
  
  // 同时备份和替换 source map 文件
  console.log('\n处理 source map 文件...');
  let mapReplacedCount = 0;
  
  minifiedFiles.forEach(minFile => {
    const originalFile = minFile.replace('.min.js', '.js');
    const mapFile = minFile + '.map';
    const originalMapPath = path.join(templatesJsDir, originalFile + '.map');
    const minifiedMapPath = path.join(distJsDir, mapFile);
    
    if (fs.existsSync(minifiedMapPath)) {
      try {
        // 备份原始 map 文件（如果存在）
        if (fs.existsSync(originalMapPath)) {
          const backupMapPath = path.join(backupDir, path.basename(originalMapPath));
          fs.copyFileSync(originalMapPath, backupMapPath);
        }
        
        // 复制新的 map 文件
        fs.copyFileSync(minifiedMapPath, originalMapPath);
        console.log(`  ✓ 替换: ${originalFile}.map`);
        mapReplacedCount++;
      } catch (error) {
        console.log(`  ✗ 失败: ${originalFile}.map - ${error.message}`);
      }
    }
  });
  
  console.log('\n=== 替换完成 ===');
  console.log(`成功替换: ${replacedCount} 个 JS 文件`);
  console.log(`替换 source map: ${mapReplacedCount} 个文件`);
  console.log(`跳过: ${skippedCount} 个文件`);
  console.log(`\n备份位置: ${backupDir}`);
  console.log('\n提示: 如需恢复原始文件，请从 backups/original-js/ 目录复制回来');
}

replaceWithMinified();
