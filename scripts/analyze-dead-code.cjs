#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const JS_DIR = path.join(__dirname, '../templates/js');

// 分析所有 JS 文件中的函数、变量、类
function analyzeFile(filePath) {
    const content = fs.readFileSync(filePath, 'utf-8');
    const stats = {
        path: filePath,
        functions: [],
        variables: [],
        classes: [],
        calls: [],
        references: []
    };

    // 匹配函数定义（包括 async function）
    const functionMatches = content.matchAll(
        /(?:async\s+)?function\s+(\w+)\s*\(/g
    );
    for (const match of functionMatches) {
        stats.functions.push({ name: match[1], type: 'function' });
    }

    // 匹配箭头函数赋值 const funcName = (params) => {}
    const arrowFunctionMatches = content.matchAll(
        /const\s+(\w+)\s*=\s*(?:async\s+)?\(.*\)\s*=>/g
    );
    for (const match of arrowFunctionMatches) {
        stats.functions.push({ name: match[1], type: 'arrow-function' });
    }

    // 匹配方法定义
    const methodMatches = content.matchAll(
        /(\w+)\s*\([^)]*\)\s*\{/g
    );
    for (const match of methodMatches) {
        if (!['if', 'for', 'while', 'switch', 'catch', 'function'].includes(match[1])) {
            stats.functions.push({ name: match[1], type: 'method' });
        }
    }

    // 匹配变量声明 const/let/var
    const varMatches = content.matchAll(
        /(?:const|let|var)\s+(\w+)\s*=/g
    );
    for (const match of varMatches) {
        stats.variables.push(match[1]);
    }

    // 匹配类定义
    const classMatches = content.matchAll(
        /class\s+(\w+)/g
    );
    for (const match of classMatches) {
        stats.classes.push(match[1]);
    }

    // 匹配函数调用
    const callMatches = content.matchAll(
        /(\w+)\s*\(/g
    );
    for (const match of callMatches) {
        stats.calls.push(match[1]);
    }

    // 匹配属性访问 obj.prop
    const propMatches = content.matchAll(
        /\.(\w+)/g
    );
    for (const match of propMatches) {
        stats.references.push(match[1]);
    }

    return stats;
}

// 读取所有 JS 文件
function getAllJsFiles(dir) {
    const files = [];
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    
    for (const entry of entries) {
        if (entry.isFile() && entry.name.endsWith('.js')) {
            files.push(path.join(dir, entry.name));
        }
    }
    
    return files;
}

// 主分析函数
function main() {
    console.log('🔍 JavaScript 死代码分析\n');
    console.log('=' .repeat(60));

    const jsFiles = getAllJsFiles(JS_DIR);
    const allStats = [];

    // 分析每个文件
    for (const file of jsFiles) {
        const stats = analyzeFile(file);
        allStats.push(stats);
    }

    // 汇总所有定义和引用
    const allFunctions = new Set();
    const allVariables = new Set();
    const allCalls = new Set();
    const allReferences = new Set();

    for (const stats of allStats) {
        stats.functions.forEach(f => allFunctions.add(f.name));
        stats.variables.forEach(v => allVariables.add(v));
        stats.calls.forEach(c => allCalls.add(c));
        stats.references.forEach(r => allReferences.add(r));
    }

    // 识别未使用的函数
    console.log('\n📊 可能未使用的函数 (定义但未调用):\n');
    let unusedCount = 0;
    
    for (const stats of allStats) {
        const fileName = path.basename(stats.path);
        const unused = stats.functions.filter(f => 
            !allCalls.has(f.name) && 
            !allReferences.has(f.name) &&
            f.name !== 'constructor' &&
            f.name !== 'init' &&
            f.name !== 'DOMContentLoaded'
        );

        if (unused.length > 0) {
            console.log(`  📄 ${fileName}:`);
            unused.forEach(f => {
                console.log(`    ❌ ${f.name} (${f.type})`);
                unusedCount++;
            });
        }
    }

    if (unusedCount === 0) {
        console.log('  ✅ 未发现明显的未使用函数');
    }

    // 识别未使用的变量
    console.log('\n📊 可能未使用的变量 (定义但未引用):\n');
    let unusedVarCount = 0;

    for (const stats of allStats) {
        const fileName = path.basename(stats.path);
        const unused = stats.variables.filter(v => 
            !allCalls.has(v) && 
            !allReferences.has(v) &&
            v.length > 1 && // 排除单字母变量
            v !== 'require' &&
            v !== 'module' &&
            v !== 'exports'
        );

        if (unused.length > 0 && unused.length < 20) { // 只显示少量结果
            console.log(`  📄 ${fileName}:`);
            unused.slice(0, 10).forEach(v => {
                console.log(`    ⚠️  ${v}`);
                unusedVarCount++;
            });
            if (unused.length > 10) {
                console.log(`    ... 还有 ${unused.length - 10} 个`);
            }
        }
    }

    if (unusedVarCount === 0) {
        console.log('  ✅ 未发现明显的未使用变量');
    }

    // 文件大小分析
    console.log('\n📊 文件大小分析:\n');
    const sizeStats = jsFiles.map(file => {
        const stats = fs.statSync(file);
        return {
            name: path.basename(file),
            size: stats.size,
            functions: allStats.find(s => s.path === file).functions.length
        };
    }).sort((a, b) => b.size - a.size);

    sizeStats.forEach(item => {
        const sizeKB = (item.size / 1024).toFixed(2);
        console.log(`  📄 ${item.name.padEnd(35)} ${sizeKB.padStart(8)} KB  (${item.functions} 函数)`);
    });

    // 总结
    console.log('\n' + '='.repeat(60));
    console.log('\n📈 总结:\n');
    console.log(`  总文件数: ${jsFiles.length}`);
    console.log(`  总函数数: ${allFunctions.size}`);
    console.log(`  可能未使用的函数: ${unusedCount}`);
    console.log(`  可能未使用的变量: ${unusedVarCount}`);
    console.log(`  总文件大小: ${(sizeStats.reduce((sum, item) => sum + item.size, 0) / 1024).toFixed(2)} KB`);
    
    console.log('\n💡 建议:\n');
    console.log('  1. 检查标记为 ❌ 的函数是否真的未使用');
    console.log('  2. 使用 Terser 或 Closure Compiler 的 tree-shaking 功能');
    console.log('  3. 考虑使用 TypeScript 进行更严格的类型检查');
    console.log('  4. 使用 ESLint 检测未使用的变量和函数');
}

main();
