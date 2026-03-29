#!/usr/bin/env node

/**
 * 终极压缩脚本 - 使用 Google Closure Compiler + 多策略组合
 */

const fs = require('fs');
const path = require('path');
const { minify: terserMinify } = require('terser');
const { execSync } = require('child_process');

// 配置
const JS_DIR = path.join(__dirname, '../templates/js');
const OUTPUT_DIR = path.join(__dirname, '../static/dist/js-ultimate');
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
 * 终极 Terser 配置 - 最激进版本
 */
const ULTIMATE_TERSER_OPTIONS = {
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
    inline: 3,
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
    passes: 15,        // 增加到15轮
    unsafe: true,
    unsafe_arrows: true,
    unsafe_comps: true,
    unsafe_Function: true,
    unsafe_math: true,
    unsafe_methods: true,
    unsafe_proto: true,
    unsafe_regexp: true,
    unsafe_undefined: true,
    module: false,
    toplevel: true,    // 优化顶层作用域
    keep_classnames: false,
    keep_fargs: false,  // 删除未使用的函数参数
    keep_fnames: false,
  },
  mangle: {
    toplevel: true,
    eval: true,
    keep_classnames: false,
    keep_fnames: false,
    module: false,
    safari10: false,
    properties: {
      regex: /^_/,      // 混淆所有 _ 开头的属性
      reserved: [],
      undeclared: false, // 混淆未声明的属性
    },
  },
  format: {
    comments: false,
    ascii_only: false,
    beautify: false,
    braces: false,
    indent_level: 0,
    indent_start: 0,
    inline_script: true,
    max_line_len: false,  // 强制单行输出
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
  toplevel: true,
};

/**
 * Google Closure Compiler 配置
 */
function closureCompilerCompress(code) {
  try {
    const tempInput = '/tmp/closure-input.js';
    const tempOutput = '/tmp/closure-output.js';
    
    fs.writeFileSync(tempInput, code);
    
    // 使用 Google Closure Compiler (ADVANCED_OPTIMIZATIONS)
    execSync(
      `npx google-closure-compiler --js=${tempInput} ` +
      `--compilation_level=ADVANCED_OPTIMIZATIONS ` +
      `--language_out=ECMASCRIPT_2022 ` +
      `--js_output_file=${tempOutput} ` +
      `--warning_level=QUIET`,
      { stdio: 'pipe' }
    );
    
    const compressed = fs.readFileSync(tempOutput, 'utf-8');
    
    // 清理临时文件
    fs.unlinkSync(tempInput);
    fs.unlinkSync(tempOutput);
    
    return compressed;
  } catch (error) {
    // 如果 Closure Compiler 失败，返回原代码
    return code;
  }
}

/**
 * esbuild 压缩
 */
function esbuildCompress(code) {
  try {
    const tempInput = '/tmp/esbuild-input.js';
    const tempOutput = '/tmp/esbuild-output.js';
    
    fs.writeFileSync(tempInput, code);
    
    execSync(
      `npx esbuild ${tempInput} --minify --minify-whitespace --mangle-props=^_ --tree-shaking=true --outfile=${tempOutput}`,
      { stdio: 'pipe' }
    );
    
    const compressed = fs.readFileSync(tempOutput, 'utf-8');
    
    fs.unlinkSync(tempInput);
    fs.unlinkSync(tempOutput);
    
    return compressed;
  } catch (error) {
    return code;
  }
}

/**
 * Terser 压缩
 */
function terserCompress(code, options) {
  try {
    const result = terserMinify(code, options);
    if (result.error) {
      throw result.error;
    }
    return result.code;
  } catch (error) {
    return code;
  }
}

/**
 * 字符串优化
 */
function optimizeStrings(code) {
  // 压缩重复字符串
  const stringMap = {};
  let counter = 0;
  
  code = code.replace(/(["'`])(.*?)\1/g, (match, quote, str) => {
    if (str.length > 8 && !str.includes('\\') && !str.includes(quote)) {
      if (!stringMap[str]) {
        stringMap[str] = `S${counter++}`;
      }
      return quote + stringMap[str] + quote;
    }
    return match;
  });
  
  return code;
}

/**
 * 强制单行压缩
 */
function forceSingleLine(code) {
  // 移除所有换行符
  code = code.replace(/\n/g, '');
  // 移除多余的空格
  code = code.replace(/\s+/g, ' ');
  // 移除分号前的空格
  code = code.replace(/\s*;/g, ';');
  // 移除逗号后的空格
  code = code.replace(/,\s*/g, ',');
  // 移除运算符周围的空格
  code = code.replace(/\s*([=+\-*/%&|^|!<>?:{}()\[\]])\s*/g, '$1');
  
  return code;
}

/**
 * 终极压缩流程
 */
function ultimateCompress(originalCode, fileSize) {
  let code = originalCode;
  
  if (!code) {
    return '';
  }
  
  console.log(`   开始终极压缩 (原始: ${Math.round(fileSize / 1024)} KB)`);
  
  // 对于小文件，使用轻量级压缩
  if (fileSize < 5 * 1024) {
    const result = terserCompress(code, ULTIMATE_TERSER_OPTIONS);
    if (result) {
      code = result;
      code = optimizeStrings(code);
    }
    code = forceSingleLine(code);
    return code;
  }
  
  // 对于中型文件，使用中等压缩
  if (fileSize < 50 * 1024) {
    console.log(`   第一轮: esbuild...`);
    code = esbuildCompress(code);
    console.log(`   第二轮: Terser (15轮)...`);
    for (let i = 0; i < 15; i++) {
      const result = terserCompress(code, ULTIMATE_TERSER_OPTIONS);
      if (result) {
        code = result;
      }
    }
    code = optimizeStrings(code);
    code = forceSingleLine(code);
    return code;
  }
  
  // 对于大文件，使用全部压缩工具
  console.log(`   第一轮: Google Closure Compiler...`);
  const closureResult = closureCompilerCompress(code);
  if (closureResult !== code) {
    code = closureResult;
    console.log(`   第二轮: esbuild...`);
    code = esbuildCompress(code);
  } else {
    console.log(`   Closure Compiler 不可用，使用备选方案`);
    console.log(`   第二轮: esbuild...`);
    code = esbuildCompress(code);
  }
  
  console.log(`   第三轮: Terser (20轮)...`);
  for (let i = 0; i < 20; i++) {
    const result = terserCompress(code, ULTIMATE_TERSER_OPTIONS);
    if (result) {
      code = result;
    }
  }
  
  console.log(`   第四轮: 字符串优化...`);
  code = optimizeStrings(code);
  
  console.log(`   第五轮: 强制单行压缩...`);
  code = forceSingleLine(code);
  
  return code;
}

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
 * 压缩单个文件
 */
async function compressFile(file) {
  console.log(`📄 处理: ${file.relative}`);
  
  if (isAdminFile(file.path)) {
    console.log(`⏭️  跳过 admin 文件: ${file.relative}`);
    return null;
  }
  
  try {
    const originalCode = fs.readFileSync(file.path, 'utf-8');
    const originalSize = Buffer.byteLength(originalCode, 'utf8');
    
    const compressedCode = ultimateCompress(originalCode, originalSize);
    
    const compressedSize = Buffer.byteLength(compressedCode, 'utf-8');
    const compressionRatio = ((originalSize - compressedSize) / originalSize * 100).toFixed(2);
    
    const outputPath = path.join(OUTPUT_DIR, file.relative.replace(/\.js$/, '.min.js'));
    const outputDir = path.dirname(outputPath);
    
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }
    
    fs.writeFileSync(outputPath, compressedCode, 'utf-8');
    
    console.log(`   ✅ 压缩后: ${Math.round(compressedSize / 1024)} KB (${compressionRatio}%)`);
    
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

  let report = `# 终极压缩报告\n\n生成时间: ${new Date().toISOString()}\n\n## 摘要\n\n- 总文件数: ${results.length}\n- 成功压缩: ${successful.length}\n- 失败: ${results.length - successful.length}\n\n## 性能统计\n\n- 原始总大小: ${Math.round(totalOriginalSize / 1024)} KB\n- 压缩后总大小: ${Math.round(totalCompressedSize / 1024)} KB\n- 平均压缩率: ${avgCompressionRatio}%\n\n## 压缩效果排序\n\n| 文件 | 原始大小 | 压缩后 | 压缩率 |\n|------|----------|--------|--------|\n`;

  successful.sort((a, b) => parseFloat(b.compressionRatio) - parseFloat(a.compressionRatio));
  
  successful.forEach(r => {
    report += `| \`${r.file}\` | ${Math.round(r.originalSize / 1024)} KB | ${Math.round(r.compressedSize / 1024)} KB | ${r.compressionRatio}% |\n`;
  });

  report += `\n## 压缩策略\n\n- **小文件** (< 5KB): Terser 深度压缩 + 字符串优化\n- **中型文件** (5-50KB): esbuild + Terser 15轮 + 字符串优化\n- **大文件** (> 50KB): Google Closure Compiler + esbuild + Terser 20轮 + 字符串优化\n\n## 压缩特性\n\n- **顶级混淆**: 顶层变量和函数名完全混淆\n- **属性混淆**: _ 开头的属性全部混淆\n- **多轮压缩**: 最多20轮深度压缩\n- **不安全优化**: 启用所有 unsafe 选项\n- **字符串优化**: 压缩重复的长字符串\n- **智能策略**: 根据文件大小选择最优压缩策略\n`;

  return report;
}

/**
 * 主函数
 */
async function main() {
  console.log('🚀 开始终极压缩流程...');
  console.log('');
  
  if (!fs.existsSync(OUTPUT_DIR)) {
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  }
  
  console.log('🔍 扫描所有 JS 文件...');
  const files = scanJsFiles();
  console.log(`✅ 找到 ${files.length} 个 JS 文件`);
  console.log('');
  
  console.log('📦 开始压缩文件...');
  const results = [];
  for (const file of files) {
    const result = await compressFile(file);
    results.push(result);
  }
  
  console.log('');
  console.log('📊 生成压缩报告...');
  const report = generateReport(results);
  const reportPath = path.join(OUTPUT_DIR, 'ultimate-compression-report.md');
  fs.writeFileSync(reportPath, report, 'utf-8');
  console.log(`✅ 报告已保存到: ${reportPath}`);
  
  const successful = results.filter(r => r !== null);
  const failed = results.filter(r => r === null);
  const skipped = files.filter(f => isAdminFile(f.path)).length;
  
  console.log('');
  console.log('🎉 终极压缩完成！');
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

main().catch(console.error);
