const fs = require('fs');
const path = require('path');
const { minify } = require('html-minifier-terser');
const csso = require('csso');

// HTML 压缩配置
const htmlMinifyOptions = {
  collapseWhitespace: true,
  removeComments: true,
  removeRedundantAttributes: true,
  removeScriptTypeAttributes: true,
  removeStyleLinkTypeAttributes: true,
  useShortDoctype: true,
  minifyCSS: true,
  minifyJS: true,
  quoteCharacter: '"',
  conservativeCollapse: false,
  continueOnParseError: false
};

// CSS 压缩配置
const cssMinifyOptions = {
  comments: false,
  forceMediaMerge: true
};

// 获取文件大小（格式化）
function getFileSize(filePath) {
  const stats = fs.statSync(filePath);
  const bytes = stats.size;
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

// 压缩单个 HTML 文件
async function minifyHtmlFile(filePath) {
  console.log(`\n处理: ${path.basename(filePath)}`);
  
  const originalSize = fs.statSync(filePath).size;
  const htmlContent = fs.readFileSync(filePath, 'utf8');
  
  try {
    // 先压缩内联 CSS
    let processedContent = htmlContent;
    const styleRegex = /<style>([\s\S]*?)<\/style>/gi;
    processedContent = processedContent.replace(styleRegex, (match, css) => {
      try {
        const minifiedCss = csso.minify(css, cssMinifyOptions).css;
        return `<style>${minifiedCss}</style>`;
      } catch (e) {
        console.warn(`  警告: CSS 压缩失败 - ${e.message}`);
        return match;
      }
    });

    // 压缩 HTML
    const minified = await minify(processedContent, htmlMinifyOptions);
    
    // 备份原文件
    const backupPath = `${filePath}.bak`;
    if (!fs.existsSync(backupPath)) {
      fs.copyFileSync(filePath, backupPath);
      console.log(`  ✓ 原文件已备份到: ${path.basename(backupPath)}`);
    }

    // 写入压缩后的文件
    fs.writeFileSync(filePath, minified);
    
    const newSize = fs.statSync(filePath).size;
    const savedBytes = originalSize - newSize;
    const savedPercent = ((savedBytes / originalSize) * 100).toFixed(2);
    
    console.log(`  原始大小: ${getFileSize(filePath)}`);
    console.log(`  压缩后: ${getFileSize(filePath)}`);
    console.log(`  节省: ${savedPercent}% (${savedBytes} 字节)`);
    
    return { success: true, savedPercent, savedBytes };
  } catch (error) {
    console.error(`  ✗ 压缩失败: ${error.message}`);
    return { success: false, error: error.message };
  }
}

// 主函数
async function main() {
  console.log('=== HTML 模板压缩工具 ===');
  console.log('目标目录: public/');
  
  const publicDir = path.join(__dirname, 'public');
  const htmlFiles = [
    path.join(publicDir, 'index.html'),
    path.join(publicDir, 'login.html'),
    path.join(publicDir, 'about.html'),
    path.join(publicDir, 'admin', 'index.html')
  ];

  let totalSuccess = 0;
  let totalFailed = 0;
  let totalSavedBytes = 0;

  for (const htmlFile of htmlFiles) {
    if (fs.existsSync(htmlFile)) {
      const result = await minifyHtmlFile(htmlFile);
      if (result.success) {
        totalSuccess++;
        totalSavedBytes += result.savedBytes;
      } else {
        totalFailed++;
      }
    } else {
      console.warn(`\n文件不存在: ${htmlFile}`);
      totalFailed++;
    }
  }

  console.log('\n=== 压缩汇总 ===');
  console.log(`成功: ${totalSuccess}`);
  console.log(`失败: ${totalFailed}`);
  console.log(`总共节省: ${(totalSavedBytes / 1024).toFixed(2)} KB (${totalSavedBytes} 字节)`);
  
  if (totalSuccess > 0) {
    console.log('\n提示: 原文件已备份为 *.bak');
    console.log('如需恢复，请删除压缩文件并将 .bak 文件重命名回去');
  }
}

main().catch(console.error);