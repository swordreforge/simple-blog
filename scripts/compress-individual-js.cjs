#!/usr/bin/env node

/**
 * 单文件极致压缩工具
 * 为每个 JS 文件单独进行极致压缩，排除 admin 相关文件
 * 生成 ES 模块版本的 HTML 引用
 */

const fs = require('fs');
const path = require('path');
const { minify } = require('terser');
const { glob } = require('glob');

// 配置
const PROJECT_ROOT = path.join(__dirname, '..');
const TEMPLATES_DIR = path.join(PROJECT_ROOT, 'templates');
const JS_DIR = path.join(TEMPLATES_DIR, 'js');
const OUTPUT_DIR = path.join(PROJECT_ROOT, 'static/dist/js-modules');
const ANALYSIS_REPORT = path.join(PROJECT_ROOT, 'js-analysis/js-dependencies-report.json');

// 确保输出目录存在
if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

/**
 * 检查文件是否与 admin 相关
 */
function isAdminRelated(filePath) {
  const normalizedPath = filePath.toLowerCase();
  return normalizedPath.includes('admin') ||
         normalizedPath.includes('filemanager') ||
         normalizedPath.includes('dyn-routing');
}

/**
 * 极致压缩配置 - 简化版本，兼容性更好
 */
const EXTREME_TERSER_OPTIONS = {
  compress: {
    dead_code: true,
    drop_console: true,    // 删除所有 console
    drop_debugger: true,
    conditionals: true,
    evaluate: true,
    booleans: true,
    loops: true,
    unused: true,
    hoist_funs: true,
    keep_fargs: false,     // 删除未使用的函数参数
    hoist_vars: false,
    if_return: true,
    join_vars: true,
    side_effects: true,
    reduce_vars: true,
    reduce_funcs: true,
    collapse_vars: true,
    negate_iife: true,
    sequences: true,
    switches: true,
    properties: true,
    inline: true,
    module: false,
    toplevel: false,
    ecma: 2022,
    keep_classnames: false,
    keep_fnames: false,
    typeofs: true,
    passes: 3,  // 多次压缩轮次
    pure_funcs: null,  // 可以指定已知无副作用的函数
    pure_getters: true,
  },
  mangle: {
    toplevel: false,
    eval: true,
    keep_classnames: false,
    keep_fnames: false,
    module: false,
    safari10: false,
    properties: {
      regex: /^_/,  // 混淆以 _ 开头的属性
      reserved: []
    },
    toplevel: false,
  },
  format: {
    comments: false,
    ecma: 2022,
    safari10: false,
    webkit: false,
    wrap_iife: false,
    beautify: false,
    ascii_only: false,
    indent_level: 0,
    indent_start: 0,
    max_line_len: false,
    semicolons: true,
    quote_style: 0,
    preserve_annotations: false,
  },
  sourceMap: false,
  ecma: 2022,
  keep_classnames: false,
  keep_fnames: false,
  module: false,
  toplevel: false,
  nameCache: null,
  toplevel: false,
};

/**
 * 压缩单个 JS 文件
 */
async function compressJsFile(inputPath, outputPath) {
  try {
    const code = fs.readFileSync(inputPath, 'utf-8');

    // 检查是否已经是 ES 模块
    const isModule = code.includes('import ') || code.includes('export ');

    // 添加严格模式
    let codeWithStrict = code;
    if (!code.includes('"use strict"') && !code.includes("'use strict'")) {
      codeWithStrict = `'use strict';\n${code}`;
    }

    const result = await minify(codeWithStrict, {
      ...EXTREME_TERSER_OPTIONS,
      module: isModule,  // 如果是模块，启用模块模式
    });

    if (result.error) {
      console.error(`❌ 压缩 ${inputPath} 时出错:`, result.error);
      return { success: false, error: result.error };
    }

    return { success: true, code: result.code };
  } catch (error) {
    console.error(`❌ 压缩 ${inputPath} 时出错:`, error);
    return { success: false, error };
  }
}

/**
 * 处理所有 JS 文件
 */
async function processAllJsFiles() {
  console.log('🔍 扫描所有 JS 文件...');

  // 扫描所有 JS 文件
  const jsFiles = await glob('**/*.js', {
    cwd: JS_DIR,
    ignore: ['**/node_modules/**', '**/npm/**']  // 排除 npm 包
  });

  console.log(`✅ 找到 ${jsFiles.length} 个 JS 文件`);

  const results = {
    processed: [],
    skipped: [],
    failed: []
  };

  for (const jsFile of jsFiles) {
    const inputPath = path.join(JS_DIR, jsFile);
    const relativePath = path.relative(JS_DIR, inputPath);

    // 跳过 admin 相关文件
    if (isAdminRelated(relativePath)) {
      console.log(`⏭️  跳过 admin 文件: ${relativePath}`);
      results.skipped.push({
        file: relativePath,
        reason: 'admin related'
      });
      continue;
    }

    // 创建输出路径，保持目录结构
    const outputPath = path.join(OUTPUT_DIR, relativePath.replace(/\.js$/, '.min.js'));
    const outputDir = path.dirname(outputPath);

    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }

    // 读取原始文件大小
    const originalSize = fs.statSync(inputPath).size;

    console.log(`📄 处理: ${relativePath}`);
    const compressResult = await compressJsFile(inputPath, outputPath);

    if (compressResult.success) {
      // 写入压缩后的文件
      fs.writeFileSync(outputPath, compressResult.code);

      const compressedSize = compressResult.code.length;
      const compressionRatio = ((1 - compressedSize / originalSize) * 100).toFixed(2);

      console.log(`   ✅ 原始: ${Math.round(originalSize / 1024)} KB → 压缩: ${Math.round(compressedSize / 1024)} KB (${compressionRatio}%)`);

      results.processed.push({
        file: relativePath,
        originalSize,
        compressedSize,
        compressionRatio,
        isModule: fs.readFileSync(inputPath, 'utf-8').includes('import ') || fs.readFileSync(inputPath, 'utf-8').includes('export ')
      });
    } else {
      console.log(`   ❌ 压缩失败`);
      results.failed.push({
        file: relativePath,
        error: compressResult.error.message
      });
    }
  }

  return results;
}

/**
 * 生成新的 HTML 引用
 */
function generateNewHtmlReferences(originalScripts, processedFiles) {
  const newReferences = [];

  // 确保 originalScripts 是数组
  if (!Array.isArray(originalScripts)) {
    console.log(`⚠️  脚本数据格式不正确，跳过`);
    return [];  // 返回空数组而不是空字符串
  }

  originalScripts.forEach(script => {
    const src = script.src;
    const cleanPath = src.split('?')[0].replace('/js/', '');

    // 检查是否被处理过
    const processed = processedFiles.find(p => p.file === cleanPath);

    if (processed) {
      // 生成新的引用路径
      const newSrc = `/static/dist/js-modules/${cleanPath.replace(/\.js$/, '.min.js')}`;

      // 使用 ES 模块
      if (processed.isModule || script.type === 'module') {
        newReferences.push({
          type: 'module',
          src: newSrc,
          originalSrc: src,
          defer: true
        });
      } else {
        // 使用 IIFE 封装的非模块脚本
        newReferences.push({
          type: 'classic',
          src: newSrc,
          originalSrc: src,
          defer: script.defer || true,
          async: script.async
        });
      }
    } else {
      // admin 文件保持原样
      newReferences.push({
        type: 'classic',
        src: src,
        originalSrc: src,
        defer: script.defer,
        async: script.async,
        skipCompression: true
      });
    }
  });

  return newReferences;
}

/**
 * 生成 HTML 转换脚本
 */
function generateHtmlTransformerScript(results, analysis) {
  let script = `#!/usr/bin/env node

/**
 * HTML JS 引用转换脚本
 * 自动将原始 script 标签转换为压缩后的 ES 模块引用
 */

const fs = require('fs');
const path = require('path');

const TEMPLATES_DIR = path.join(__dirname, '../templates');

// 页面转换映射
const pageTransforms = {
`;

  Object.entries(analysis.pageDependencies).forEach(([page, scripts]) => {
    const newRefs = generateNewHtmlReferences(scripts, results.processed);

    script += `  '${page}': [\n`;
    newRefs.forEach(ref => {
      if (ref.skipCompression) {
        script += `    { src: '${ref.originalSrc}', defer: ${ref.defer}, async: ${ref.async}, skip: true },\n`;
      } else if (ref.type === 'module') {
        script += `    { src: '${ref.src}', type: 'module', defer: true },\n`;
      } else {
        script += `    { src: '${ref.src}', defer: ${ref.defer}, async: ${ref.async} },\n`;
      }
    });
    script += `  ],\n`;
  });

  script += `};

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

    const oldPattern = new RegExp(\`<script[^>]*src="\${transform.originalSrc}"[^>]*></script>\`, 'g');

    if (transform.type === 'module') {
      const newTag = \`<script type="module" src="\${transform.src}" defer></script>\`;
      html = html.replace(oldPattern, newTag);
    } else {
      const newTag = \`<script src="\${transform.src}"\${transform.defer ? ' defer' : ''}\${transform.async ? ' async' : ''}></script>\`;
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
      console.log(\`📦 已备份: \${page}\`);

      // 转换
      const transformedHtml = transformHtmlFile(htmlPath, pageTransforms[page]);
      fs.writeFileSync(htmlPath, transformedHtml);
      console.log(\`✅ 已转换: \${page}\`);
    }
  });

  console.log('🎉 转换完成！');
  console.log(\`📁 备份位置: \${backupDir}\`);
}

if (require.main === module) {
  main();
}
`;

  return script;
}

/**
 * 生成报告
 */
function generateReport(results, analysis) {
  const report = {
    timestamp: new Date().toISOString(),
    summary: {
      totalFiles: results.processed.length + results.skipped.length + results.failed.length,
      processed: results.processed.length,
      skipped: results.skipped.length,
      failed: results.failed.length
    },
    stats: {
      totalOriginalSize: results.processed.reduce((sum, r) => sum + r.originalSize, 0),
      totalCompressedSize: results.processed.reduce((sum, r) => sum + r.compressedSize, 0),
      averageCompressionRatio: results.processed.length > 0
        ? (results.processed.reduce((sum, r) => sum + parseFloat(r.compressionRatio), 0) / results.processed.length).toFixed(2)
        : 0
    },
    processed: results.processed,
    skipped: results.skipped,
    failed: results.failed
  };

  // 保存 JSON 报告
  const reportPath = path.join(OUTPUT_DIR, 'compression-report.json');
  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

  // 生成 Markdown 报告
  let md = '# JS 单文件极致压缩报告\n\n';
  md += `生成时间: ${report.timestamp}\n\n`;

  md += '## 摘要\n\n';
  md += `- 总文件数: ${report.summary.totalFiles}\n`;
  md += `- 已处理: ${report.summary.processed}\n`;
  md += `- 已跳过: ${report.summary.skipped}\n`;
  md += `- 失败: ${report.summary.failed}\n\n`;

  md += '## 性能统计\n\n';
  md += `- 原始总大小: ${Math.round(report.stats.totalOriginalSize / 1024)} KB\n`;
  md += `- 压缩后总大小: ${Math.round(report.stats.totalCompressedSize / 1024)} KB\n`;
  md += `- 平均压缩率: ${report.stats.averageCompressionRatio}%\n\n`;

  md += '## 已处理文件\n\n';
  md += '| 文件 | 原始大小 | 压缩后 | 压缩率 | 模块 |\n';
  md += '|------|----------|--------|--------|------|\n';

  results.processed.forEach(file => {
    md += `| \`${file.file}\` | ${Math.round(file.originalSize / 1024)} KB | ${Math.round(file.compressedSize / 1024)} KB | ${file.compressionRatio}% | ${file.isModule ? '✅' : '❌'} |\n`;
  });

  if (results.skipped.length > 0) {
    md += '\n## 已跳过文件 (admin 相关)\n\n';
    results.skipped.forEach(file => {
      md += `- \`${file.file}\` - ${file.reason}\n`;
    });
  }

  if (results.failed.length > 0) {
    md += '\n## 失败文件\n\n';
    results.failed.forEach(file => {
      md += `- \`${file.file}\` - ${file.error}\n`;
    });
  }

  md += '\n## 使用说明\n\n';
  md += '### 1. 查看压缩结果\n';
  md += '```bash\n';
  md += 'cat static/dist/js-modules/compression-report.json\n';
  md += '```\n\n';

  md += '### 2. 转换 HTML 文件\n';
  md += '生成的转换脚本会自动将 HTML 中的 script 标签替换为压缩后的版本：\n\n';
  md += '```bash\n';
  md += 'node scripts/transform-html-references.cjs\n';
  md += '```\n\n';

  md += '### 3. 回滚\n';
  md += '如果需要回滚，原始文件已备份到 `templates/backup-original/` 目录。\n\n';

  // 保存 Markdown 报告
  const mdReportPath = path.join(OUTPUT_DIR, 'compression-report.md');
  fs.writeFileSync(mdReportPath, md);

  return report;
}

/**
 * 主函数
 */
async function main() {
  console.log('🚀 开始 JS 单文件极致压缩流程...\n');

  // 处理所有 JS 文件
  const results = await processAllJsFiles();

  // 读取分析报告
  let analysis = null;
  if (fs.existsSync(ANALYSIS_REPORT)) {
    analysis = JSON.parse(fs.readFileSync(ANALYSIS_REPORT, 'utf-8'));
  }

  // 生成 HTML 转换脚本
  console.log('\n📝 生成 HTML 转换脚本...');
  const transformerScript = generateHtmlTransformerScript(results, analysis);
  const transformerPath = path.join(PROJECT_ROOT, 'scripts/transform-html-references.cjs');
  fs.writeFileSync(transformerPath, transformerScript);
  fs.chmodSync(transformerPath, '755');
  console.log(`✅ HTML 转换脚本已保存到: ${transformerPath}`);

  // 生成报告
  console.log('\n📊 生成压缩报告...');
  const report = generateReport(results, analysis);
  console.log(`✅ 报告已保存到: ${OUTPUT_DIR}/compression-report.md`);

  console.log('\n🎉 处理完成！');
  console.log('\n📊 最终统计:');
  console.log(`   - 总文件数: ${report.summary.totalFiles}`);
  console.log(`   - 已处理: ${report.summary.processed}`);
  console.log(`   - 已跳过: ${report.summary.skipped}`);
  console.log(`   - 失败: ${report.summary.failed}`);
  console.log(`   - 原始总大小: ${Math.round(report.stats.totalOriginalSize / 1024)} KB`);
  console.log(`   - 压缩后总大小: ${Math.round(report.stats.totalCompressedSize / 1024)} KB`);
  console.log(`   - 平均压缩率: ${report.stats.averageCompressionRatio}%`);

  console.log(`\n📁 输出目录: ${OUTPUT_DIR}`);
  console.log(`\n📖 查看详细报告: ${OUTPUT_DIR}/compression-report.md`);
  console.log(`\n🔄 下一步: 运行 HTML 转换脚本`);
  console.log(`   node ${transformerPath}`);
}

// 运行主函数
if (require.main === module) {
  main().catch(console.error);
}

module.exports = {
  compressJsFile,
  processAllJsFiles,
  isAdminRelated,
  generateNewHtmlReferences
};