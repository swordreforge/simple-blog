// 测试 Web Worker 日期解析
const testArticle = {
  id: 14,
  title: 'test-article-06',
  created_at: '2026-02-02 00:00:00',
  published_at: null
};

// 测试日期解析
const dateStr = testArticle.published_at || testArticle.created_at || testArticle.date;
console.log('日期字符串:', dateStr);

const date = new Date(dateStr);
console.log('Date 对象:', date);
console.log('是否有效:', !isNaN(date.getTime()));
console.log('年份:', date.getFullYear());
console.log('月份:', date.getMonth() + 1);