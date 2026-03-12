#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const JS_DIR = path.join(__dirname, '../templates/js');

console.log('📊 JavaScript Bundle 分析\n');
console.log('='.repeat(80));

// 分析文件大小
function analyzeBundle() {
    const files = fs.readdirSync(JS_DIR)
        .filter(f => f.endsWith('.js') && !f.includes('.min'))
        .map(f => {
            const filePath = path.join(JS_DIR, f);
            const stats = fs.statSync(filePath);
            const content = fs.readFileSync(filePath, 'utf-8');
            
            // 统计行数和字符数
            const lines = content.split('\n').length;
            const chars = content.length;
            
            // 估算压缩后大小 (约 40-50% 压缩率)
            const estimatedMinified = Math.round(chars * 0.45);
            
            return {
                name: f,
                size: stats.size,
                lines,
                chars,
                estimatedMinified
            };
        })
        .sort((a, b) => b.size - a.size);

    // 显示详细信息
    console.log('\n📁 文件大小详情:\n');
    
    let totalSize = 0;
    let totalEstimatedMinified = 0;
    
    files.forEach((file, index) => {
        const sizeKB = (file.size / 1024).toFixed(2);
        const minifiedKB = (file.estimatedMinified / 1024).toFixed(2);
        const reduction = ((1 - file.estimatedMinified / file.size) * 100).toFixed(1);
        
        totalSize += file.size;
        totalEstimatedMinified += file.estimatedMinified;
        
        const rank = (index + 1).toString().padStart(2);
        const bar = '█'.repeat(Math.min(50, Math.floor(file.size / 2000)));
        
        console.log(`  ${rank}. ${file.name}`);
        console.log(`     原始: ${sizeKB.padStart(8)} KB | 行数: ${file.lines.toString().padStart(5)} | 字符: ${file.chars}`);
        console.log(`     压缩: ${minifiedKB.padStart(8)} KB (${reduction}%)`);
        console.log(`     ${bar}`);
        console.log();
    });

    // 分类统计
    console.log('📊 分类统计:\n');
    
    const categories = {
        'Admin 文件': files.filter(f => f.name.includes('admin')),
        'UI 组件': files.filter(f => 
            f.name.includes('focus') || 
            f.name.includes('modal') || 
            f.name.includes('floating')
        ),
        '功能模块': files.filter(f => 
            f.name.includes('music') || 
            f.name.includes('login') || 
            f.name.includes('filemanager') ||
            f.name.includes('keyboard')
        ),
        '第三方库': files.filter(f => 
            f.name.includes('chart') || 
            f.name.includes('highlight')
        ),
    };

    for (const [category, items] of Object.entries(categories)) {
        if (items.length > 0) {
            const catSize = items.reduce((sum, f) => sum + f.size, 0);
            const catMinified = items.reduce((sum, f) => sum + f.estimatedMinified, 0);
            const percentage = ((catSize / totalSize) * 100).toFixed(1);
            
            console.log(`  ${category}:`);
            console.log(`    文件数: ${items.length}`);
            console.log(`    总大小: ${(catSize / 1024).toFixed(2)} KB (${percentage}%)`);
            console.log(`    压缩后: ${(catMinified / 1024).toFixed(2)} KB`);
            console.log();
        }
    }

    // 优化建议
    console.log('💡 优化建议:\n');
    
    const largeFiles = files.filter(f => f.size > 50000);
    if (largeFiles.length > 0) {
        console.log('  🚨 大文件 (>50KB) 需要优化:');
        largeFiles.forEach(f => {
            console.log(`    - ${f.name}: ${(f.size / 1024).toFixed(2)} KB`);
        });
        console.log();
    }

    const adminFiles = categories['Admin 文件'];
    if (adminFiles.length > 3) {
        console.log('  📋 Admin 文件过多，建议:');
        console.log('    1. 按功能拆分 (文章管理、用户管理、评论管理等)');
        console.log('    2. 使用动态导入 (import()) 按需加载');
        console.log();
    }

    if (categories['第三方库'].length > 0) {
        console.log('  📚 第三方库优化:');
        categories['第三方库'].forEach(f => {
            console.log(`    - ${f.name}: ${(f.size / 1024).toFixed(2)} KB`);
            console.log(`      建议: 使用 CDN 或按需导入`);
        });
        console.log();
    }

    // 总结
    console.log('='.repeat(80));
    console.log('\n📈 总结:\n');
    console.log(`  总文件数: ${files.length}`);
    console.log(`  总大小: ${(totalSize / 1024).toFixed(2)} KB`);
    console.log(`  估算压缩后: ${(totalEstimatedMinified / 1024).toFixed(2)} KB`);
    console.log(`  预计减少: ${((1 - totalEstimatedMinified / totalSize) * 100).toFixed(1)}%`);
    console.log();
    
    // 生成可视化报告
    generateReport(files, totalSize);
}

function generateReport(files, totalSize) {
    const reportPath = path.join(__dirname, '../static/dist/bundle-report.html');
    
    const html = `<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>JavaScript Bundle 分析报告</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #333; margin-bottom: 10px; }
        .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 30px 0; }
        .stat-card { background: #f8f9fa; padding: 20px; border-radius: 8px; text-align: center; }
        .stat-value { font-size: 2em; font-weight: bold; color: #007bff; }
        .stat-label { color: #666; margin-top: 5px; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; }
        th { background: #f8f9fa; font-weight: 600; }
        tr:hover { background: #f8f9fa; }
        .size-bar { height: 20px; background: #e9ecef; border-radius: 10px; overflow: hidden; }
        .size-fill { height: 100%; background: linear-gradient(90deg, #007bff, #00d4ff); transition: width 0.3s; }
        .percentage { color: #666; font-size: 0.9em; }
    </style>
</head>
<body>
    <div class="container">
        <h1>📊 JavaScript Bundle 分析报告</h1>
        <p style="color: #666; margin-bottom: 20px;">生成时间: ${new Date().toLocaleString('zh-CN')}</p>
        
        <div class="summary">
            <div class="stat-card">
                <div class="stat-value">${files.length}</div>
                <div class="stat-label">文件总数</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">${(totalSize / 1024).toFixed(1)} KB</div>
                <div class="stat-label">总大小</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">${((files.reduce((sum, f) => sum + f.estimatedMinified, 0) / 1024).toFixed(1))} KB</div>
                <div class="stat-label">估算压缩后</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">${((1 - files.reduce((sum, f) => sum + f.estimatedMinified, 0) / totalSize) * 100).toFixed(1)}%</div>
                <div class="stat-label">压缩率</div>
            </div>
        </div>

        <h2 style="margin-top: 30px;">📁 文件详情</h2>
        <table>
            <thead>
                <tr>
                    <th>排名</th>
                    <th>文件名</th>
                    <th>原始大小</th>
                    <th>行数</th>
                    <th>占比</th>
                </tr>
            </thead>
            <tbody>
                ${files.map((file, index) => {
                    const percentage = ((file.size / totalSize) * 100).toFixed(1);
                    return `
                        <tr>
                            <td>${index + 1}</td>
                            <td>${file.name}</td>
                            <td>${(file.size / 1024).toFixed(2)} KB</td>
                            <td>${file.lines.toLocaleString()}</td>
                            <td style="width: 30%;">
                                <div class="size-bar">
                                    <div class="size-fill" style="width: ${percentage}%"></div>
                                </div>
                                <span class="percentage">${percentage}%</span>
                            </td>
                        </tr>
                    `;
                }).join('')}
            </tbody>
        </table>
    </div>
</body>
</html>`;

    fs.writeFileSync(reportPath, html);
    console.log(`✅ HTML 报告已生成: ${reportPath}`);
}

analyzeBundle();