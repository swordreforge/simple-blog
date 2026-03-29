#!/usr/bin/env node

/**
 * HTML JS 引用转换脚本
 * 自动将原始 script 标签转换为压缩后的 ES 模块引用
 */

const fs = require('fs');
const path = require('path');

const TEMPLATES_DIR = path.join(__dirname, '../templates');

// 页面转换映射
const pageTransforms = {
  'passage.html': [
  ],
  'markdown-editor.html': [
  ],
  'index.html': [
  ],
  'friends.html': [
  ],
  'collect.html': [
  ],
  'about.html': [
  ],
  'admin/filemanager.html': [
  ],
  'admin/dyn-routing.html': [
  ],
  'admin/admin.html': [
  ],
};

/**
 * 转换单个 HTML 文件
 */
function transformHtmlFile(htmlPath, transforms) {
  let html = fs.readFileSync(htmlPath, 'utf-8');

  // 按照原始顺序替换 script 标签
  transforms.forEach(transform => {
    if (transform.skip) {
      // admin 文件保持不变
      return;
    }

    const oldPattern = new RegExp(`<script[^>]*src="${transform.originalSrc}"[^>]*></script>`, 'g');

    if (transform.type === 'module') {
      const newTag = `<script type="module" src="${transform.src}" defer></script>`;
      html = html.replace(oldPattern, newTag);
    } else {
      const newTag = `<script src="${transform.src}"${transform.defer ? ' defer' : ''}${transform.async ? ' async' : ''}></script>`;
      html = html.replace(oldPattern, newTag);
    }
  });

  return html;
}

/**
 * 主函数
 */
function main() {
  console.log('🔄 开始转换 HTML 文件...');

  // 备份原始文件
  const backupDir = path.join(TEMPLATES_DIR, 'backup-original');
  if (!fs.existsSync(backupDir)) {
    fs.mkdirSync(backupDir, { recursive: true });
  }

  Object.keys(pageTransforms).forEach(page => {
    const htmlPath = path.join(TEMPLATES_DIR, page);
    const backupPath = path.join(backupDir, page);

    if (fs.existsSync(htmlPath)) {
      // 备份
      fs.copyFileSync(htmlPath, backupPath);
      console.log(`📦 已备份: ${page}`);

      // 转换
      const transformedHtml = transformHtmlFile(htmlPath, pageTransforms[page]);
      fs.writeFileSync(htmlPath, transformedHtml);
      console.log(`✅ 已转换: ${page}`);
    }
  });

  console.log('🎉 转换完成！');
  console.log(`📁 备份位置: ${backupDir}`);
}

if (require.main === module) {
  main();
}
