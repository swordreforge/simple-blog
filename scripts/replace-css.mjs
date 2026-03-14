import { readFileSync, writeFileSync, readdirSync, copyFileSync, mkdirSync, statSync } from 'fs'
import { resolve, dirname, join } from 'path'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const templatesDir = resolve(__dirname, '../templates')
const distCssDir = resolve(__dirname, '../static/dist/css')
const backupsDir = resolve(__dirname, '../backups/original-css')

// 确保备份目录存在
try {
  mkdirSync(backupsDir, { recursive: true })
} catch (err) {
  // 目录已存在
}

// CSS文件映射（从optimize-css.js的输出获取）
const cssMapping = {
  'animations.css': 'animations-ep7c1j67.css',
  'dark-mode.css': 'dark-mode-6k5cjoqy.css',
  'filemanager.css': 'filemanager-wbc07eik.css',
  'floating-text-mb0u05wn.css': 'floating-text-mb0u05wn-lk1xue2f.css',
  'floating-text.css': 'floating-text-mb0u05wn-lk1xue2f.css',
  'glass-effect.css': 'glass-effect-5nabn5ud.css',
  'katex.min.css': 'katex.min-n2idu7xh.css',
  'keyboard-shortcuts.css': 'keyboard-shortcuts-rjyvng9y.css',
  'modal-animations.css': 'modal-animations-rs0lh9hd.css',
  'music-player.css': 'music-player-9ph6hrjy.css',
  'passage-base.css': 'passage-base-1jlekls6.css',
  'passage.css': 'passage-base-1jlekls6.css',
  'settings.css': 'settings-5w8gdsnv.css',
  'tokyo-night-dark.min.css': 'tokyo-night-dark.min-69j7ug5y.css'
}

// 递归获取所有HTML文件
function getHtmlFiles(dir, fileList = []) {
  const files = readdirSync(dir, { withFileTypes: true })
  for (const file of files) {
    const filePath = join(dir, file.name)
    if (file.isDirectory()) {
      getHtmlFiles(filePath, fileList)
    } else if (file.name.endsWith('.html')) {
      fileList.push(filePath)
    }
  }
  return fileList
}

// 替换HTML文件中的CSS引用
function replaceCssReferences(filePath) {
  let content = readFileSync(filePath, 'utf-8')
  let modified = false

  for (const [original, minified] of Object.entries(cssMapping)) {
    // 匹配 href="/css/original.css" 或 href='/css/original.css'
    const regex = new RegExp(`href=["']([^"']*${original.replace('.', '\\.')})["']`, 'g')

    if (regex.test(content)) {
      content = content.replace(regex, (match, href) => {
        // 保留完整的路径，只替换文件名
        const pathParts = href.split('/')
        pathParts[pathParts.length - 1] = minified
        return `href="/static/dist/css/${pathParts.join('/')}"` // 使用 /static/dist/css/ 路径
      })
      modified = true
    }
  }

  if (modified) {
    writeFileSync(filePath, content, 'utf-8')
    return true
  }
  return false
}

// 主函数
async function main() {
  console.log('开始替换 CSS 文件...\n')

  // 获取所有HTML文件
  const htmlFiles = getHtmlFiles(templatesDir)
  console.log(`找到 ${htmlFiles.length} 个 HTML 文件\n`)

  let replacedCount = 0

  for (const htmlFile of htmlFiles) {
    if (replaceCssReferences(htmlFile)) {
      replacedCount++
      const fileName = htmlFile.split('/').pop()
      console.log(`  ✓ 替换: ${fileName}`)
    }
  }

  console.log(`\n=== 替换完成 ===`)
  console.log(`成功替换: ${replacedCount} 个 HTML 文件`)
  console.log(`\n提示: 原始CSS文件已备份到 ${backupsDir}`)
}

main().catch(console.error)