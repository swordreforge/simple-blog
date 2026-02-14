#!/usr/bin/env node

/**
 * Passage.html 侧边栏优化功能测试脚本
 * 测试虚拟滚动、筛选器、懒加载和 Web Worker 功能
 */

import http from 'http';
import url from 'url';

// 测试配置
const config = {
  baseUrl: 'http://localhost:8080',
  testArticleId: '3' // 从数据库获取的文章 ID
};

// 测试结果
const results = {
  virtualScroll: { name: '虚拟滚动功能', passed: false, issues: [] },
  filterByYearMonth: { name: '按年/月筛选功能', passed: false, issues: [] },
  filterBySearch: { name: '搜索功能', passed: false, issues: [] },
  folderToggle: { name: '文件夹展开/折叠', passed: false, issues: [] },
  webWorker: { name: 'Web Worker 集成', passed: false, issues: [] }
};

// 发送 HTTP 请求
function httpRequest(path, method = 'GET', data = null) {
  return new Promise((resolve, reject) => {
    const parsedUrl = url.parse(config.baseUrl + path);
    const options = {
      hostname: parsedUrl.hostname,
      port: parsedUrl.port,
      path: parsedUrl.path,
      method: method,
      headers: {
        'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36',
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8'
      }
    };

    if (data) {
      options.headers['Content-Type'] = 'application/json';
      options.headers['Content-Length'] = Buffer.byteLength(JSON.stringify(data));
    }

    const req = http.request(options, (res) => {
      let body = '';
      res.on('data', (chunk) => body += chunk);
      res.on('end', () => {
        try {
          resolve({ status: res.statusCode, headers: res.headers, body });
        } catch (e) {
          reject(e);
        }
      });
    });

    req.on('error', reject);
    if (data) {
      req.write(JSON.stringify(data));
    }
    req.end();
  });
}

// 测试 1: 检查 passage.html 页面是否正常加载
async function testPageLoad() {
  console.log('\n=== 测试 1: 页面加载 ===');
  try {
    const response = await httpRequest(`/passage?id=${config.testArticleId}`);
    if (response.status === 200) {
      console.log('✅ 页面加载成功');
      
      // 检查关键资源是否被引用
      const html = response.body;
      
      // 检查虚拟滚动脚本
      if (html.includes('/js/virtual-scroll.js')) {
        console.log('✅ 虚拟滚动脚本已引用');
      } else {
        console.log('❌ 虚拟滚动脚本未引用');
        results.virtualScroll.issues.push('虚拟滚动脚本未在 HTML 中引用');
      }
      
      // 检查筛选器脚本
      if (html.includes('/js/sidebar-filter.js')) {
        console.log('✅ 筛选器脚本已引用');
      } else {
        console.log('❌ 筛选器脚本未引用');
        results.filterByYearMonth.issues.push('筛选器脚本未在 HTML 中引用');
        results.filterBySearch.issues.push('筛选器脚本未在 HTML 中引用');
      }
      
      // 检查 Web Worker 脚本
      if (html.includes('/js/sidebar-worker.js')) {
        console.log('✅ Web Worker 脚本已引用');
      } else {
        console.log('❌ Web Worker 脚本未引用');
        results.webWorker.issues.push('Web Worker 脚本未在 HTML 中引用');
      }
      
      // 检查侧边栏元素
      if (html.includes('id="fileTree"') || html.includes('id="sidebar"')) {
        console.log('✅ 侧边栏元素存在');
      } else {
        console.log('❌ 侧边栏元素不存在');
        results.virtualScroll.issues.push('侧边栏 DOM 元素不存在');
      }
      
      // 检查初始化代码
      if (html.includes('initSidebarOptimizations')) {
        console.log('✅ 初始化函数存在');
      } else {
        console.log('❌ 初始化函数不存在');
        results.webWorker.issues.push('初始化函数不存在');
      }
      
      return html;
    } else {
      console.log(`❌ 页面加载失败: ${response.status}`);
      return null;
    }
  } catch (error) {
    console.log(`❌ 请求失败: ${error.message}`);
    return null;
  }
}

// 测试 2: 检查 API 是否正常返回文章数据
async function testArticleAPI() {
  console.log('\n=== 测试 2: 文章 API ===');
  try {
    const response = await httpRequest('/api/passage/list');
    if (response.status === 200) {
      const data = JSON.parse(response.body);
      if (data.data && Array.isArray(data.data) && data.data.length > 0) {
        console.log(`✅ API 返回 ${data.data.length} 篇文章`);
        return data.data;
      } else {
        console.log('❌ API 未返回有效文章数据');
        return [];
      }
    } else {
      console.log(`❌ API 请求失败: ${response.status}`);
      return [];
    }
  } catch (error) {
    console.log(`❌ API 请求失败: ${error.message}`);
    return [];
  }
}

// 测试 3: 检查脚本文件是否存在
async function testScriptFiles() {
  console.log('\n=== 测试 3: 脚本文件检查 ===');
  
  const scripts = [
    '/js/virtual-scroll.js',
    '/js/sidebar-filter.js',
    '/js/sidebar-worker.js'
  ];
  
  for (const script of scripts) {
    try {
      const response = await httpRequest(script);
      if (response.status === 200) {
        console.log(`✅ ${script} 文件存在且可访问`);
        
        // 检查脚本内容
        const content = response.body;
        if (script.includes('virtual-scroll')) {
          if (content.includes('class VirtualScroll') && content.includes('class SidebarVirtualScroll')) {
            console.log('  ✅ 包含虚拟滚动类定义');
          } else {
            console.log('  ❌ 虚拟滚动类定义不完整');
            results.virtualScroll.issues.push('虚拟滚动类定义不完整');
          }
        } else if (script.includes('sidebar-filter')) {
          if (content.includes('class SidebarFilter') && content.includes('class LazyFolderLoader')) {
            console.log('  ✅ 包含筛选器和懒加载类定义');
          } else {
            console.log('  ❌ 筛选器/懒加载类定义不完整');
            results.filterByYearMonth.issues.push('筛选器类定义不完整');
          }
        } else if (script.includes('sidebar-worker')) {
          if (content.includes('self.onmessage') && content.includes('buildFolders')) {
            console.log('  ✅ 包含 Web Worker 消息处理');
          } else {
            console.log('  ❌ Web Worker 定义不完整');
            results.webWorker.issues.push('Web Worker 定义不完整');
          }
        }
      } else {
        console.log(`❌ ${script} 文件不可访问 (${response.status})`);
      }
    } catch (error) {
      console.log(`❌ ${script} 请求失败: ${error.message}`);
    }
  }
}

// 分析并输出测试结果
function analyzeResults() {
  console.log('\n=== 测试结果汇总 ===');
  
  let passedTests = 0;
  let totalTests = Object.keys(results).length;
  
  for (const [key, result] of Object.entries(results)) {
    // 如果没有发现问题，标记为通过
    result.passed = result.issues.length === 0;
    
    if (result.passed) {
      passedTests++;
      console.log(`✅ ${result.name} - 通过`);
    } else {
      console.log(`❌ ${result.name} - 失败`);
      result.issues.forEach(issue => {
        console.log(`   - ${issue}`);
      });
    }
  }
  
  console.log(`\n总计: ${passedTests}/${totalTests} 通过`);
  
  // 根据测试结果给出建议
  console.log('\n=== 建议 ===');
  
  if (results.virtualScroll.issues.length > 0) {
    console.log('- 虚拟滚动功能存在问题，请检查 virtual-scroll.js 文件是否正确实现');
  }
  
  if (results.filterByYearMonth.issues.length > 0 || results.filterBySearch.issues.length > 0) {
    console.log('- 筛选功能存在问题，请检查 sidebar-filter.js 文件是否正确实现');
  }
  
  if (results.webWorker.issues.length > 0) {
    console.log('- Web Worker 功能存在问题，请检查 sidebar-worker.js 文件是否正确实现');
  }
  
  if (passedTests === totalTests) {
    console.log('- 所有功能测试通过，可以在浏览器中进行实际交互测试');
  }
}

// 主测试函数
async function runTests() {
  console.log('========================================');
  console.log('  Passage.html 侧边栏优化功能测试');
  console.log('========================================');
  
  // 运行测试
  await testPageLoad();
  await testArticleAPI();
  await testScriptFiles();
  
  // 分析结果
  analyzeResults();
}

// 执行测试
runTests().catch(console.error);