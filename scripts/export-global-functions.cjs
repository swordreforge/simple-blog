const fs = require('fs');

const filePath = 'templates/js/admin-inline-1.js';
let content = fs.readFileSync(filePath, 'utf-8');

// 需要导出的函数
const functionsToExport = [
  { name: 'showToast', line: 1 },
  { name: 'openModal', line: 1467 },
  { name: 'closeModal', line: 1471 }
];

functionsToExport.forEach(({ name, line }) => {
  const regex = new RegExp(`^function ${name}\\(`, 'gm');
  const match = regex.exec(content);

  if (match) {
    const fullMatch = match[0];
    const replacement = fullMatch.replace(`function ${name}`, `window.${name} = function ${name}`);
    content = content.replace(fullMatch, replacement);
    console.log(`✓ 导出: ${name}`);
  }
});

fs.writeFileSync(filePath, content, 'utf-8');
console.log('\n✅ 已更新 admin-inline-1.js');