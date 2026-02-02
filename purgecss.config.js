import { glob } from 'glob'

// 获取所有 HTML 文件
const htmlFiles = glob.sync('templates/**/*.html')

// 获取所有 JS 文件
const jsFiles = glob.sync('templates/**/*.js')

export default {
  // 要扫描的文件
  content: [
    ...htmlFiles,
    ...jsFiles
  ],
  // 要处理的 CSS 文件
  css: [
    'templates/css/**/*.css',
    'templates/**/*.css'
  ],
  // 要排除的类名（动态生成的）
  safelist: {
    // 保留所有以 data- 开头的类
    pattern: /^data-/,
    // 保留所有 :hover, :active, :focus 等伪类
    standard: [/^hover:/, /^focus:/, /^active:/, /^visited:/, /^disabled:/],
    // 保留关键帧动画
    keyframes: [
      'fadeIn',
      'fadeOut',
      'slideIn',
      'slideOut',
      'bounce',
      'pulse',
      'spin',
      'shake',
      'zoomIn',
      'zoomOut',
      'flip',
      'rotate'
    ],
    // 保留特定类名（用于动态生成的）
    greedy: [
      /^modal-/,
      /^toast-/,
      /^shortcut-/,
      /^shortcut-hint/,
      /^article-/,
      /^sidebar-/,
      /^file-/,
      /^filter-/,
      /^tab-/,
      /^comment-/,
      /^sponsor-/,
      /^attachment-/,
      /^preview-/,
      /^fm-/,
      /^music-/,
      /^active$/,
      /^hidden$/,
      /^show$/
    ]
  },
  // 动态类名提取
  defaultExtractor: (content) => {
    // 提取所有类名
    return content.match(/[\w-/:]+(?<!:)/g) || []
  },
  // 输出文件
  output: 'static/dist/css/purged.css',
  // 统计信息
  rejected: true,
  rejectedCss: true
}