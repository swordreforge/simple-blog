#!/usr/bin/env node

/**
 * 超级极致压缩脚本 - 比之前的更加激进
 * 使用多工具组合 + 多轮压缩 + 高级混淆
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const { minify: terserMinify } = require('terser');

// 配置
const JS_DIR = path.join(__dirname, '../templates/js');
const OUTPUT_DIR = path.join(__dirname, '../static/dist/js-ultra');
const ADMIN_FILES = [
  'filemanager.js',
  'dyn-routing.js',
  'admin-inline.js',
  'admin-inline-1.js',
  'admin-inline-2.js',
  'admin-inline-4.js',
  'admin-4730.js'
];

/**
 * 超级激进的 Terser 配置
 */
const ULTRA_TERSER_OPTIONS = {
  compress: {
    dead_code: true,
    drop_console: true,
    drop_debugger: true,
    conditionals: true,
    evaluate: true,
    booleans: true,
    loops: true,
    unused: true,
    hoist_funs: true,
    hoist_props: true,
    hoist_vars: false,
    if_return: true,
    inline: 3,          // 更激进的内联
    join_vars: true,
    side_effects: true,
    reduce_vars: true,
    reduce_funcs: true,
    collapse_vars: true,
    negate_iife: true,
    sequences: true,
    switches: true,
    properties: true,
    comparisons: true,
    computed_props: true,
    typeofs: true,
    passes: 10,         // 10轮压缩
    unsafe: true,       // 启用不安全优化
    unsafe_arrows: true,
    unsafe_comps: true,
    unsafe_Function: true,
    unsafe_math: true,
    unsafe_methods: true,
    unsafe_proto: true,
    unsafe_regexp: true,
    unsafe_undefined: true,
  },
  mangle: {
    toplevel: true,     // 混淆顶层变量
    eval: true,
    keep_classnames: false,
    keep_fnames: false,
    module: false,
    safari10: false,
    properties: {
      regex: /^_/,      // 混淆所有 _ 开头的属性
      reserved: []
    },
  },
  format: {
    comments: false,    // 删除所有注释
    ascii_only: false,
    beautify: false,
    braces: false,
    indent_level: 0,
    indent_start: 0,
    inline_script: true,
    max_line_len: 32000,
    preserve_annotations: false,
    quote_keys: false,
    quote_style: 0,
    semicolons: true,
    shebang: true,
    wrap_func_args: true,
    wrap_iife: true,
  },
  ecma: 2022,
  sourceMap: false,
  toplevel: true,      // 优化顶层作用域
};

/**
 * 扫描所有 JS 文件
 */
function scanJsFiles() {
  const files = [];
  
  function scanDir(dir, prefix = '') {
    const items = fs.readdirSync(dir);
    for (const item of items) {
      const fullPath = path.join(dir, item);
      const stat = fs.statSync(fullPath);
      
      if (stat.isDirectory()) {
        scanDir(fullPath, path.join(prefix, item));
      } else if (item.endsWith('.js') && !item.endsWith('.min.js')) {
        const relativePath = path.join(prefix, item);
        files.push({
          path: fullPath,
          relative: relativePath,
          size: stat.size
        });
      }
    }
  }
  
  scanDir(JS_DIR);
  return files;
}

/**
 * 检查是否是 admin 文件
 */
function isAdminFile(filePath) {
  const fileName = path.basename(filePath);
  return ADMIN_FILES.some(adminFile => fileName.includes(adminFile));
}

/**
 * 第一轮：使用 esbuild 进行初步压缩
 */
function esbuildCompress(code) {
  try {
    const tempInput = '/tmp/input.js';
    const tempOutput = '/tmp/output.js';
    
    fs.writeFileSync(tempInput, code);
    
    execSync(`npx esbuild ${tempInput} --minify --mangle-props=^_ --tree-shaking=true --outfile=${tempOutput}`, {
      stdio: 'pipe'
    });
    
    const compressed = fs.readFileSync(tempOutput, 'utf-8');
    
    // 清理临时文件
    fs.unlinkSync(tempInput);
    fs.unlinkSync(tempOutput);
    
    return compressed;
  } catch (error) {
    console.error('esbuild 压缩失败:', error.message);
    return code;
  }
}

/**
 * 第二轮：使用 Terser 进行深度压缩
 */
function terserCompress(code, options) {
  try {
    const result = terserMinify(code, options);
    if (result.error) {
      throw result.error;
    }
    return result.code;
  } catch (error) {
    console.error('Terser 压缩失败:', error.message);
    return code;
  }
}

/**
 * 后处理：进一步优化
 */
function postProcess(code) {
  // 移除多余的空行
  code = code.replace(/\n\s*\n/g, '\n');
  
  // 移除多余的空格
  code = code.replace(/;\s*}/g, '}');
  code = code.replace(/\{\s*;/g, '{');
  
  // 压缩字符串
  code = code.replace(/'([^']+)'/g, (match, str) => {
    if (!str.includes('\\') && !str.includes("'")) {
      return `'${str}'`;
    }
    return match;
  });
  
  return code;
}

/**
 * 多轮压缩
 */
function multiRoundCompress(originalCode) {
  let code = originalCode;
  const rounds = 5;
  
  for (let i = 0; i < rounds; i++) {
    console.log(`   第 ${i + 1}/${rounds} 轮压缩...`);
    
    // 使用 esbuild 进行压缩
    code = esbuildCompress(code);
    
    // 使用 Terser 进行深度压缩
    const result = terserCompress(code, ULTRA_TERSER_OPTIONS);
    if (result) {
      code = result;
    }
    
    // 后处理
    code = postProcess(code);
    
    console.log(`   第 ${i + 1} 轮完成，当前大小: ${Math.round(code.length / 1024)} KB`);
  }
  
  return code;
}

/**
 * 压缩单个文件
 */
async function compressFile(file) {
  console.log(`📄 处理: ${file.relative}`);
  
  // 跳过 admin 文件
  if (isAdminFile(file.path)) {
    console.log(`⏭️  跳过 admin 文件: ${file.relative}`);
    return null;
  }
  
  try {
    // 读取原始代码
    const originalCode = fs.readFileSync(file.path, 'utf-8');
    const originalSize = Buffer.byteLength(originalCode, 'utf8');
    
    // 多轮压缩
    const compressedCode = multiRoundCompress(originalCode);
    
    const compressedSize = Buffer.byteLength(compressedCode, 'utf-8');
    const compressionRatio = ((originalSize - compressedSize) / originalSize * 100).toFixed(2);
    
    // 生成输出文件路径
    const outputPath = path.join(OUTPUT_DIR, file.relative.replace(/\.js$/, '.min.js'));
    const outputDir = path.dirname(outputPath);
    
    // 确保输出目录存在
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }
    
    // 写入压缩后的文件
    fs.writeFileSync(outputPath, compressedCode, 'utf-8');
    
    console.log(`   ✅ 原始: ${Math.round(originalSize / 1024)} KB → 压缩: ${Math.round(compressedSize / 1024)} KB (${compressionRatio}%)`);
    
    return {
      file: file.relative,
      originalSize,
      compressedSize,
      compressionRatio
    };
  } catch (error) {
    console.error(`   ❌ 压缩失败: ${error.message}`);
    return null;
  }
}

/**
 * 生成报告
 */
function generateReport(results) {
  const successful = results.filter(r => r !== null);
  const totalOriginalSize = successful.reduce((sum, r) => sum + r.originalSize, 0);
  const totalCompressedSize = successful.reduce((sum, r) => sum + r.compressedSize, 0);
  const avgCompressionRatio = successful.length > 0 
    ? ((totalOriginalSize - totalCompressedSize) / totalOriginalSize * 100).toFixed(2)
    : '0.00';

  let report = `# 超级极致压缩报告

生成时间: ${new Date().toISOString()}

## 摘要

- 总文件数: ${results.length}
- 成功压缩: ${successful.length}
- 失败: ${results.length - successful.length}

## 性能统计

- 原始总大小: ${Math.round(totalOriginalSize / 1024)} KB
- 压缩后总大小: ${Math.round(totalCompressedSize / 1024)} KB
- 平均压缩率: ${avgCompressionRatio}%

## 压缩效果排序

| 文件 | 原始大小 | 压缩后 | 压缩率 |
|------|----------|--------|--------|
`;

  // 按压缩率排序
  successful.sort((a, b) => parseFloat(b.compressionRatio) - parseFloat(a.compressionRatio));
  
  successful.forEach(r => {
    report += `| \`${r.file}\` | ${Math.round(r.originalSize / 1024)} KB | ${Math.round(r.compressedSize / 1024)} KB | ${r.compressionRatio}% |\n`;
  });

  report += `
## 压缩特性

- **多轮压缩**: 5轮压缩 + esbuild + Terser
- **深度混淆**: 顶层变量混淆 + 属性混淆
- **不安全优化**: 启用所有 unsafe 选项
- **后处理**: 额外的优化步骤
- **ES2022 支持**: 使用最新语法特性
`;

  return report;
}

/**
 * 主函数
 */
async function main() {
  console.log('🚀 开始超级极致压缩流程...');
  console.log('');
  
  // 创建输出目录
  if (!fs.existsSync(OUTPUT_DIR)) {
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  }
  
  // 扫描所有 JS 文件
  console.log('🔍 扫描所有 JS 文件...');
  const files = scanJsFiles();
  console.log(`✅ 找到 ${files.length} 个 JS 文件`);
  console.log('');
  
  // 压缩所有文件
  console.log('📦 开始压缩文件...');
  const results = [];
  for (const file of files) {
    const result = await compressFile(file);
    results.push(result);
  }
  
  console.log('');
  
  // 生成报告
  console.log('📊 生成压缩报告...');
  const report = generateReport(results);
  const reportPath = path.join(OUTPUT_DIR, 'ultra-compression-report.md');
  fs.writeFileSync(reportPath, report, 'utf-8');
  console.log(`✅ 报告已保存到: ${reportPath}`);
  
  // 统计
  const successful = results.filter(r => r !== null);
  const failed = results.filter(r => r === null);
  const skipped = files.filter(f => isAdminFile(f.path)).length;
  
  console.log('');
  console.log('🎉 压缩完成！');
  console.log('');
  console.log('📊 最终统计:');
  console.log(`   - 总文件数: ${files.length}`);
  console.log(`   - 成功压缩: ${successful.length}`);
  console.log(`   - 已跳过: ${skipped}`);
  console.log(`   - 失败: ${failed.length}`);
  
  if (successful.length > 0) {
    const totalOriginalSize = successful.reduce((sum, r) => sum + r.originalSize, 0);
    const totalCompressedSize = successful.reduce((sum, r) => sum + r.compressedSize, 0);
    const avgCompressionRatio = ((totalOriginalSize - totalCompressedSize) / totalOriginalSize * 100).toFixed(2);
    
    console.log(`   - 原始总大小: ${Math.round(totalOriginalSize / 1024)} KB`);
    console.log(`   - 压缩后总大小: ${Math.round(totalCompressedSize / 1024)} KB`);
    console.log(`   - 平均压缩率: ${avgCompressionRatio}%`);
  }
  
  console.log('');
  console.log(`📁 输出目录: ${OUTPUT_DIR}`);
  console.log(`📖 查看详细报告: ${reportPath}`);
}

// 运行主函数
main().catch(console.error);
