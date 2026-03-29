#!/usr/bin/env node

/**
 * JS 依赖分析工具
 * 扫描所有 HTML 文件，提取 JS 引用，分析依赖关系
 */

const fs = require('fs');
const path = require('path');
const { glob } = require('glob');

// 配置
const TEMPLATES_DIR = path.join(__dirname, '../templates');
const OUTPUT_DIR = path.join(__dirname, '../js-analysis');
const JS_DIR = path.join(TEMPLATES_DIR, 'js');

// 确保输出目录存在
if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

/**
 * 从 HTML 内容中提取所有 script 标签的 src 属性
 */
function extractScriptSources(htmlContent) {
  const scriptRegex = /<script[^>]*src=["']([^"']+)["'][^>]*>/gi;
  const sources = [];
  let match;

  while ((match = scriptRegex.exec(htmlContent)) !== null) {
    sources.push({
      src: match[1],
      type: match[0].includes('type="module"') ? 'module' : 'classic',
      defer: match[0].includes('defer'),
      async: match[0].includes('async'),
      fullMatch: match[0]
    });
  }

  return sources;
}

/**
 * 扫描所有 HTML 文件
 */
async function scanAllHtmlFiles() {
  const htmlFiles = await glob('**/*.html', {
    cwd: TEMPLATES_DIR,
    ignore: ['**/node_modules/**']
  });

  const pageDependencies = {};

  for (const htmlFile of htmlFiles) {
    const htmlPath = path.join(TEMPLATES_DIR, htmlFile);
    const htmlContent = fs.readFileSync(htmlPath, 'utf-8');
    const scriptSources = extractScriptSources(htmlContent);

    // 过滤掉内联脚本（没有 src 属性的）
    const externalScripts = scriptSources.filter(s => s.src && !s.src.startsWith('data:'));

    if (externalScripts.length > 0) {
      pageDependencies[htmlFile] = externalScripts;
    }
  }

  return pageDependencies;
}

/**
 * 分析依赖模式
 */
function analyzeDependencies(pageDependencies) {
  const allScripts = new Map(); // script src -> { pages: Set<string>, type: string }
  const pageSpecificScripts = new Map(); // page -> Set<script src>
  const sharedScripts = new Map(); // script src -> Set<page names>

  // 收集所有脚本引用
  Object.entries(pageDependencies).forEach(([page, scripts]) => {
    pageSpecificScripts.set(page, new Set());

    scripts.forEach(script => {
      const src = script.src;

      if (!allScripts.has(src)) {
        allScripts.set(src, {
          pages: new Set(),
          type: script.type,
          defer: script.defer,
          async: script.async
        });
      }

      allScripts.get(src).pages.add(page);
      pageSpecificScripts.get(page).add(src);

      if (!sharedScripts.has(src)) {
        sharedScripts.set(src, new Set());
      }
      sharedScripts.get(src).add(page);
    });
  });

  // 找出真正共享的脚本（被 2 个或更多页面引用）
  const trulySharedScripts = new Map();
  sharedScripts.forEach((pages, src) => {
    if (pages.size >= 2) {
      trulySharedScripts.set(src, pages);
    }
  });

  // 找出页面特定的脚本
  const pageOnlyScripts = new Map();
  pageSpecificScripts.forEach((scripts, page) => {
    const onlyForThisPage = new Set();
    scripts.forEach(src => {
      if (sharedScripts.get(src).size === 1) {
        onlyForThisPage.add(src);
      }
    });
    if (onlyForThisPage.size > 0) {
      pageOnlyScripts.set(page, onlyForThisPage);
    }
  });

  return {
    allScripts,
    pageDependencies,
    sharedScripts: trulySharedScripts,
    pageOnlyScripts,
    stats: {
      totalPages: Object.keys(pageDependencies).length,
      totalUniqueScripts: allScripts.size,
      sharedScriptsCount: trulySharedScripts.size,
      pageSpecificScriptsCount: Array.from(pageOnlyScripts.values()).reduce((sum, set) => sum + set.size, 0)
    }
  };
}

/**
 * 生成分析报告
 */
function generateReport(analysis) {
  const report = {
    timestamp: new Date().toISOString(),
    summary: analysis.stats,
    sharedScripts: {},
    pageDependencies: {},
    recommendations: []
  };

  // 共享脚本详情
  analysis.sharedScripts.forEach((pages, src) => {
    report.sharedScripts[src] = {
      usedBy: Array.from(pages),
      type: analysis.allScripts.get(src).type,
      defer: analysis.allScripts.get(src).defer,
      async: analysis.allScripts.get(src).async
    };
  });

  // 每个页面的依赖
  Object.entries(analysis.pageDependencies).forEach(([page, scripts]) => {
    report.pageDependencies[page] = {
      totalScripts: scripts.length,
      sharedScripts: scripts.filter(s => analysis.sharedScripts.has(s.src)).map(s => s.src),
      pageSpecificScripts: scripts.filter(s => !analysis.sharedScripts.has(s.src)).map(s => s.src),
      details: scripts
    };
  });

  // 生成建议
  const recommendations = [];

  // 建议 1: 提取共享脚本
  if (analysis.sharedScripts.size > 0) {
    const mostShared = Array.from(analysis.sharedScripts.entries())
      .sort((a, b) => b[1].size - a[1].size)
      .slice(0, 5);

    recommendations.push({
      type: 'extraction',
      priority: 'high',
      title: '提取共享脚本为公共模块',
      description: '以下脚本被多个页面共享，建议提取为公共模块',
      scripts: mostShared.map(([src, pages]) => ({
        src,
        usedBy: Array.from(pages).length,
        pages: Array.from(pages)
      }))
    });
  }

  // 建议 2: 合并页面特定脚本
  const pagesWithManyScripts = Object.entries(analysis.pageDependencies)
    .filter(([_, scripts]) => scripts.length > 3)
    .map(([page, scripts]) => ({ page, count: scripts.length }));

  if (pagesWithManyScripts.length > 0) {
    recommendations.push({
      type: 'merging',
      priority: 'medium',
      title: '合并页面特定脚本',
      description: '以下页面引用了多个脚本，建议合并为单个文件以减少 HTTP 请求',
      pages: pagesWithManyScripts
    });
  }

  // 建议 3: 异步加载优化
  const scriptsWithLoadOptimization = Array.from(analysis.allScripts.entries())
    .filter(([_, info]) => !info.defer && !info.async)
    .map(([src, _]) => src);

  if (scriptsWithLoadOptimization.length > 0) {
    recommendations.push({
      type: 'performance',
      priority: 'low',
      title: '考虑使用 defer/async 优化加载',
      description: '以下脚本没有使用 defer 或 async，考虑添加这些属性以优化页面加载性能',
      scripts: scriptsWithLoadOptimization
    });
  }

  report.recommendations = recommendations;

  return report;
}

/**
 * 主函数
 */
async function main() {
  console.log('🔍 开始扫描 HTML 文件...');

  const pageDependencies = await scanAllHtmlFiles();
  console.log(`✅ 扫描完成，找到 ${Object.keys(pageDependencies).length} 个 HTML 文件`);

  console.log('📊 分析依赖关系...');
  const analysis = analyzeDependencies(pageDependencies);
  console.log(`✅ 分析完成`);
  console.log(`   - 总页面数: ${analysis.stats.totalPages}`);
  console.log(`   - 唯一脚本数: ${analysis.stats.totalUniqueScripts}`);
  console.log(`   - 共享脚本数: ${analysis.stats.sharedScriptsCount}`);
  console.log(`   - 页面特定脚本数: ${analysis.stats.pageSpecificScriptsCount}`);

  console.log('📝 生成报告...');
  const report = generateReport(analysis);

  // 保存 JSON 报告
  const reportPath = path.join(OUTPUT_DIR, 'js-dependencies-report.json');
  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
  console.log(`✅ 报告已保存到: ${reportPath}`);

  // 保存 Markdown 报告
  const mdReport = generateMarkdownReport(report);
  const mdReportPath = path.join(OUTPUT_DIR, 'js-dependencies-report.md');
  fs.writeFileSync(mdReportPath, mdReport);
  console.log(`✅ Markdown 报告已保存到: ${mdReportPath}`);

  console.log('\n📋 主要发现:');
  report.recommendations.forEach((rec, index) => {
    console.log(`\n${index + 1}. ${rec.title} (${rec.priority})`);
    console.log(`   ${rec.description}`);
  });

  return report;
}

/**
 * 生成 Markdown 报告
 */
function generateMarkdownReport(report) {
  let md = '# JS 依赖分析报告\n\n';
  md += `生成时间: ${report.timestamp}\n\n`;

  md += '## 摘要\n\n';
  md += `- 总页面数: ${report.summary.totalPages}\n`;
  md += `- 唯一脚本数: ${report.summary.totalUniqueScripts}\n`;
  md += `- 共享脚本数: ${report.summary.sharedScriptsCount}\n`;
  md += `- 页面特定脚本数: ${report.summary.pageSpecificScriptsCount}\n\n`;

  md += '## 共享脚本\n\n';
  md += '| 脚本路径 | 使用页面数 | 使用页面 | 类型 | defer | async |\n';
  md += '|---------|----------|---------|------|-------|-------|\n';

  Object.entries(report.sharedScripts).forEach(([src, info]) => {
    md += `| \`${src}\` | ${info.usedBy.length} | ${info.usedBy.join(', ')} | ${info.type} | ${info.defer} | ${info.async} |\n`;
  });

  md += '\n## 页面依赖详情\n\n';

  Object.entries(report.pageDependencies).forEach(([page, info]) => {
    md += `### ${page}\n\n`;
    md += `- 总脚本数: ${info.totalScripts}\n`;
    md += `- 共享脚本数: ${info.sharedScripts.length}\n`;
    md += `- 页面特定脚本数: ${info.pageSpecificScripts.length}\n\n`;

    if (info.sharedScripts.length > 0) {
      md += '**共享脚本:**\n';
      info.sharedScripts.forEach(src => {
        md += `- \`${src}\`\n`;
      });
      md += '\n';
    }

    if (info.pageSpecificScripts.length > 0) {
      md += '**页面特定脚本:**\n';
      info.pageSpecificScripts.forEach(src => {
        md += `- \`${src}\`\n`;
      });
      md += '\n';
    }

    md += '**所有脚本:**\n';
    info.details.forEach(script => {
      md += `- \`${script.src}\` (type: ${script.type}, defer: ${script.defer}, async: ${script.async})\n`;
    });
    md += '\n';
  });

  md += '## 优化建议\n\n';

  report.recommendations.forEach((rec, index) => {
    md += `### ${index + 1}. ${rec.title} (${rec.priority})\n\n`;
    md += `${rec.description}\n\n`;

    if (rec.scripts) {
      md += '**涉及的脚本:**\n';
      rec.scripts.forEach(script => {
        if (script.usedBy) {
          md += `- \`${script.src}\` - 被 ${script.usedBy} 个页面使用\n`;
        } else {
          md += `- \`${script}\`\n`;
        }
      });
      md += '\n';
    }

    if (rec.pages) {
      md += '**涉及的页面:**\n';
      rec.pages.forEach(p => {
        md += `- \`${p.page}\` - ${p.count} 个脚本\n`;
      });
      md += '\n';
    }
  });

  return md;
}

// 运行主函数
if (require.main === module) {
  main().catch(console.error);
}

module.exports = {
  extractScriptSources,
  scanAllHtmlFiles,
  analyzeDependencies,
  generateReport
};
