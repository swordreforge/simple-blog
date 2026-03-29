#!/usr/bin/env node

/**
 * JS 合并与极致压缩工具
 * 为每个页面创建独立的、作用域封闭的 JS 文件，并进行极致压缩
 */

const fs = require('fs');
const path = require('path');
const { minify } = require('terser');

// 配置
const PROJECT_ROOT = path.join(__dirname, '..');
const TEMPLATES_DIR = path.join(PROJECT_ROOT, 'templates');
const JS_DIR = path.join(TEMPLATES_DIR, 'js');
const OUTPUT_DIR = path.join(PROJECT_ROOT, 'static/dist/js-merged');
const ANALYSIS_REPORT = path.join(PROJECT_ROOT, 'js-analysis/js-dependencies-report.json');

// 确保输出目录存在
if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

/**
 * Terser 极致压缩配置
 */
const TERSER_OPTIONS = {
  compress: {
    dead_code: true,
    drop_console: false, // 保留 console，便于调试
    drop_debugger: true,
    conditionals: true,
    evaluate: true,
    booleans: true,
    loops: true,
    unused: true,
    hoist_funs: true,
    keep_fargs: false,
    hoist_vars: false,
    if_return: true,
    join_vars: true,
    side_effects: true,
    reduce_vars: true,
    passes: 3, // 多次压缩以获得更好的效果
    module: false,
    toplevel: false,
    ecma: 2020,
    keep_classnames: false,
    keep_fnames: false,
    typeofs: true,
  },
  mangle: {
    toplevel: false, // 不混淆顶层变量
    eval: true,
    keep_classnames: false,
    keep_fnames: false,
    module: false,
    safari10: false,
    properties: {
      regex: /^_/, // 混淆以 _ 开头的属性
      reserved: []
    }
  },
  format: {
    comments: false, // 删除所有注释
    ecma: 2020,
    safari10: false,
    webkit: false,
    wrap_iife: false, // 我们手动添加 IIFE
    beautify: false,
    ascii_only: false,
    indent_level: 0,
    indent_start: 0,
    max_line_len: false,
    semicolons: true,
  },
  sourceMap: false,
  ecma: 2020,
  keep_classnames: false,
  keep_fnames: false,
  module: false,
  toplevel: false,
};

/**
 * 读取 JS 文件内容
 */
function readJsFile(filePath) {
  // 移除版本号参数（如 ?v=2）
  const cleanPath = filePath.split('?')[0];
  // 移除 /js/ 前缀
  const relativePath = cleanPath.replace('/js/', '');
  const fullPath = path.join(JS_DIR, relativePath);

  if (fs.existsSync(fullPath)) {
    return fs.readFileSync(fullPath, 'utf-8');
  }
  console.warn(`⚠️  文件不存在: ${fullPath} (原始路径: ${filePath})`);
  return '';
}

/**
 * 创建 IIFE 封装
 */
function wrapInIIFE(code, pageName) {
  return `// Page: ${pageName}
// Generated: ${new Date().toISOString()}
(function() {
'use strict';

${code}

})();`;
}

/**
 * 合并多个 JS 文件
 */
function mergeJsFiles(scripts, pageName) {
  let mergedCode = '';
  const fileOrder = [];

  scripts.forEach((script, index) => {
    const src = script.src;
    const jsContent = readJsFile(src);

    if (jsContent) {
      // 添加文件分隔注释
      mergedCode += `\n// === File ${index + 1}: ${src} ===\n`;
      mergedCode += jsContent;
      mergedCode += '\n';
      fileOrder.push(src);
    }
  });

  return { code: mergedCode, fileOrder };
}

/**
 * 使用 Terser 压缩代码
 */
async function compressCode(code, pageName) {
  try {
    const result = await minify(code, TERSER_OPTIONS);

    if (result.error) {
      console.error(`❌ 压缩 ${pageName} 时出错:`, result.error);
      return { success: false, error: result.error };
    }

    return { success: true, code: result.code };
  } catch (error) {
    console.error(`❌ 压缩 ${pageName} 时出错:`, error);
    return { success: false, error };
  }
}

/**
 * 处理单个页面
 */
async function processPage(pageName, scripts) {
  console.log(`\n📄 处理页面: ${pageName}`);
  console.log(`   - 脚本数量: ${scripts.length}`);

  // 合并 JS 文件
  const { code: mergedCode, fileOrder } = mergeJsFiles(scripts, pageName);
  console.log(`   - 合并后大小: ${Math.round(mergedCode.length / 1024)} KB`);

  // 用 IIFE 封装
  const wrappedCode = wrapInIIFE(mergedCode, pageName);

  // 压缩
  console.log(`   - 开始压缩...`);
  const compressResult = await compressCode(wrappedCode, pageName);

  if (!compressResult.success) {
    console.error(`   ❌ 压缩失败`);
    return null;
  }

  const compressedSize = compressResult.code.length;
  const originalSize = wrappedCode.length;
  const compressionRatio = ((1 - compressedSize / originalSize) * 100).toFixed(2);

  console.log(`   ✅ 压缩完成`);
  console.log(`   - 压缩后大小: ${Math.round(compressedSize / 1024)} KB`);
  console.log(`   - 压缩率: ${compressionRatio}%`);

  // 保存文件
  const outputFileName = `${pageName.replace(/\//g, '-')}.min.js`;
  const outputPath = path.join(OUTPUT_DIR, outputFileName);
  fs.writeFileSync(outputPath, compressResult.code);

  // 保存源映射信息（用于调试）
  const sourceMapInfo = {
    page: pageName,
    outputFileName,
    fileOrder,
    originalSize,
    compressedSize,
    compressionRatio,
    scripts: scripts.map(s => ({
      src: s.src,
      type: s.type,
      defer: s.defer,
      async: s.async
    }))
  };

  // 保存元数据
  const metadataPath = path.join(OUTPUT_DIR, `${outputFileName}.meta.json`);
  fs.writeFileSync(metadataPath, JSON.stringify(sourceMapInfo, null, 2));

  return sourceMapInfo;
}

/**
 * 处理共享脚本
 */
async function processSharedScripts(sharedScripts) {
  console.log('\n🔄 处理共享脚本...');

  const sharedScriptGroups = {};

  // 按使用页面数量分组
  Object.entries(sharedScripts).forEach(([src, info]) => {
    const usageCount = info.usedBy.length;
    if (!sharedScriptGroups[usageCount]) {
      sharedScriptGroups[usageCount] = [];
    }
    sharedScriptGroups[usageCount].push({ src, info });
  });

  // 创建共享脚本包
  const results = [];

  for (const [usageCount, scripts] of Object.entries(sharedScriptGroups)) {
    const groupName = `shared-${usageCount}pages`;
    console.log(`\n📦 创建共享包: ${groupName} (${usageCount} 个页面使用)`);

    const mergedCode = [];
    scripts.forEach(({ src }) => {
      const jsContent = readJsFile(src);
      if (jsContent) {
        mergedCode.push(`// === ${src} ===\n${jsContent}`);
      }
    });

    const code = mergedCode.join('\n\n');
    const wrappedCode = wrapInIIFE(code, groupName);

    console.log(`   - 开始压缩...`);
    const compressResult = await compressCode(wrappedCode, groupName);

    if (compressResult.success) {
      const outputFileName = `${groupName}.min.js`;
      const outputPath = path.join(OUTPUT_DIR, outputFileName);
      fs.writeFileSync(outputPath, compressResult.code);

      const metadata = {
        group: groupName,
        usageCount: parseInt(usageCount),
        scripts: scripts.map(s => s.src),
        originalSize: wrappedCode.length,
        compressedSize: compressResult.code.length,
        compressionRatio: ((1 - compressResult.code.length / wrappedCode.length) * 100).toFixed(2)
      };

      const metadataPath = path.join(OUTPUT_DIR, `${outputFileName}.meta.json`);
      fs.writeFileSync(metadataPath, JSON.stringify(metadata, null, 2));

      console.log(`   ✅ 完成: ${outputFileName}`);
      results.push(metadata);
    }
  }

  return results;
}

/**
 * 生成新的 HTML 引用
 */
function generateNewHtmlReferences(pageName, outputFileName) {
  return `<script src="/static/dist/js-merged/${outputFileName}" defer></script>`;
}

/**
 * 生成迁移指南
 */
function generateMigrationGuide(pageResults, sharedResults) {
  let guide = '# JS 合并与压缩迁移指南\n\n';
  guide += `生成时间: ${new Date().toISOString()}\n\n`;

  guide += '## 页面迁移\n\n';

  pageResults.forEach(result => {
    if (result) {
      guide += `### ${result.page}\n\n`;
      guide += `**原引用:**\n\`\`\`html\n`;
      result.scripts.forEach(script => {
        guide += `<script src="${script.src}"${script.defer ? ' defer' : ''}${script.async ? ' async' : ''}></script>\n`;
      });
      guide += `\`\`\`\n\n`;

      guide += `**新引用:**\n\`\`\`html\n`;
      guide += `${generateNewHtmlReferences(result.page, result.outputFileName)}\n`;
      guide += `\`\`\`\n\n`;

      guide += `**优化效果:**\n`;
      guide += `- 原始大小: ${Math.round(result.originalSize / 1024)} KB\n`;
      guide += `- 压缩后: ${Math.round(result.compressedSize / 1024)} KB\n`;
      guide += `- 压缩率: ${result.compressionRatio}%\n`;
      guide += `- 文件数量: ${result.scripts.length} → 1\n\n`;
    }
  });

  guide += '## 共享脚本包\n\n';

  sharedResults.forEach(result => {
    guide += `### ${result.group}\n\n`;
    guide += `**包含的脚本:**\n`;
    result.scripts.forEach(src => {
      guide += `- \`${src}\`\n`;
    });
    guide += `\n**使用页面数:** ${result.usageCount}\n`;
    guide += `**优化效果:**\n`;
    guide += `- 原始大小: ${Math.round(result.originalSize / 1024)} KB\n`;
    guide += `- 压缩后: ${Math.round(result.compressedSize / 1024)} KB\n`;
    guide += `- 压缩率: ${result.compressionRatio}%\n\n`;
  });

  guide += '## 使用建议\n\n';

  guide += '### 方案 1: 完全迁移（推荐）\n';
  guide += '每个页面使用独立的合并文件，最大化隔离性和压缩效果。\n\n';

  guide += '### 方案 2: 混合方案\n';
  guide += '高频使用的共享脚本提取为公共包，页面特定脚本合并。\n\n';

  guide += '### 方案 3: 渐进式迁移\n';
  guide += '先迁移高流量页面，观察效果后再逐步推广。\n\n';

  guide += '## 注意事项\n\n';
  guide += '1. **作用域隔离**: 所有代码都使用 IIFE 封装，避免全局变量污染\n';
  guide += '2. **变量名混淆**: Terser 会混淆内部变量名，减小文件体积\n';
  guide += '3. **加载顺序**: 使用 defer 属性，确保 DOM 解析完成后执行\n';
  guide += '4. **调试**: 元数据文件包含源文件映射信息，便于调试\n';
  guide += '5. **回滚**: 保留原始文件，可以随时回滚\n\n';

  guide += '## 性能对比\n\n';

  const totalOriginalSize = pageResults.reduce((sum, r) => sum + (r ? r.originalSize : 0), 0);
  const totalCompressedSize = pageResults.reduce((sum, r) => sum + (r ? r.compressedSize : 0), 0);
  const totalCompressionRatio = ((1 - totalCompressedSize / totalOriginalSize) * 100).toFixed(2);

  guide += '| 指标 | 原始 | 优化后 | 改善 |\n';
  guide += '|------|------|--------|------|\n';
  guide += `| 总大小 | ${Math.round(totalOriginalSize / 1024)} KB | ${Math.round(totalCompressedSize / 1024)} KB | ${totalCompressionRatio}% |\n`;
  guide += `| HTTP 请求数 | ${pageResults.reduce((sum, r) => sum + (r ? r.scripts.length : 0), 0)} | ${pageResults.length} | -${pageResults.reduce((sum, r) => sum + (r ? r.scripts.length : 0), 0) - pageResults.length} |\n\n`;

  return guide;
}

/**
 * 主函数
 */
async function main() {
  console.log('🚀 开始 JS 合并与压缩流程...\n');

  // 读取分析报告
  if (!fs.existsSync(ANALYSIS_REPORT)) {
    console.error('❌ 分析报告不存在，请先运行分析脚本');
    return;
  }

  const analysis = JSON.parse(fs.readFileSync(ANALYSIS_REPORT, 'utf-8'));

  console.log(`📊 分析报告信息:`);
  console.log(`   - 总页面数: ${analysis.summary.totalPages}`);
  console.log(`   - 唯一脚本数: ${analysis.summary.totalUniqueScripts}`);
  console.log(`   - 共享脚本数: ${analysis.summary.sharedScriptsCount}`);
  console.log(`   - 页面特定脚本数: ${analysis.summary.pageSpecificScriptsCount}`);

  // 处理每个页面
  const pageResults = [];

  for (const [pageName, pageData] of Object.entries(analysis.pageDependencies)) {
    const result = await processPage(pageName, pageData.details);
    if (result) {
      pageResults.push(result);
    }
  }

  // 处理共享脚本
  const sharedResults = await processSharedScripts(analysis.sharedScripts);

  // 生成迁移指南
  console.log('\n📝 生成迁移指南...');
  const migrationGuide = generateMigrationGuide(pageResults, sharedResults);
  const guidePath = path.join(OUTPUT_DIR, 'migration-guide.md');
  fs.writeFileSync(guidePath, migrationGuide);
  console.log(`✅ 迁移指南已保存到: ${guidePath}`);

  // 生成总结报告
  const summary = {
    timestamp: new Date().toISOString(),
    stats: {
      totalPagesProcessed: pageResults.length,
      totalSharedPackages: sharedResults.length,
      totalOriginalSize: pageResults.reduce((sum, r) => sum + (r ? r.originalSize : 0), 0),
      totalCompressedSize: pageResults.reduce((sum, r) => sum + (r ? r.compressedSize : 0), 0),
      overallCompressionRatio: ((1 - pageResults.reduce((sum, r) => sum + (r ? r.compressedSize : 0), 0) / pageResults.reduce((sum, r) => sum + (r ? r.originalSize : 0), 0)) * 100).toFixed(2)
    },
    pages: pageResults,
    sharedPackages: sharedResults
  };

  const summaryPath = path.join(OUTPUT_DIR, 'summary.json');
  fs.writeFileSync(summaryPath, JSON.stringify(summary, null, 2));
  console.log(`✅ 总结报告已保存到: ${summaryPath}`);

  console.log('\n🎉 处理完成！');
  console.log('\n📊 最终统计:');
  console.log(`   - 处理页面数: ${summary.stats.totalPagesProcessed}`);
  console.log(`   - 共享包数: ${summary.stats.totalSharedPackages}`);
  console.log(`   - 原始总大小: ${Math.round(summary.stats.totalOriginalSize / 1024)} KB`);
  console.log(`   - 压缩后总大小: ${Math.round(summary.stats.totalCompressedSize / 1024)} KB`);
  console.log(`   - 总体压缩率: ${summary.stats.overallCompressionRatio}%`);

  console.log(`\n📁 输出目录: ${OUTPUT_DIR}`);
  console.log(`\n📖 查看迁移指南: ${guidePath}`);
}

// 运行主函数
if (require.main === module) {
  main().catch(console.error);
}

module.exports = {
  readJsFile,
  wrapInIIFE,
  mergeJsFiles,
  compressCode,
  processPage,
  processSharedScripts,
  generateMigrationGuide
};