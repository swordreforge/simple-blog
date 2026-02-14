// 简单测试 Web Worker
console.log('开始测试 Web Worker...');

// 模拟 API 返回的数据
const mockArticles = [
  { id: 1, title: 'Test 1', created_at: '2026-02-01 00:00:00' },
  { id: 2, title: 'Test 2', created_at: '2026-02-02 00:00:00' },
  { id: 3, title: 'Test 3', created_at: '2026-01-15 00:00:00' }
];

// 创建 Worker
const worker = new Worker('/js/sidebar-worker.js');

worker.addEventListener('message', function(e) {
  console.log('收到 Worker 消息:', e.data);

  if (e.data.type === 'buildFolders' && e.data.success) {
    console.log('✅ Worker 成功构建文件夹结构');
    console.log('文件夹数量:', e.data.data.length);
    if (e.data.data.length > 0) {
      console.log('第一个文件夹:', e.data.data[0]);
    }
  } else if (e.data.type === 'buildFolders' && !e.data.success) {
    console.log('❌ Worker 构建文件夹失败');
  }
});

worker.addEventListener('error', function(e) {
  console.error('Worker 错误:', e);
});

// 发送测试数据
console.log('发送测试数据到 Worker...');
worker.postMessage({
  type: 'buildFolders',
  data: { articles: mockArticles }
});

console.log('测试已启动，等待 Worker 响应...');