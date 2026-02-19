import { readFileSync, writeFileSync, readdirSync, mkdirSync, statSync } from 'fs'
import { resolve, dirname, join } from 'path'
import { fileURLToPath } from 'url'
import postcss from 'postcss'
import autoprefixer from 'autoprefixer'
import cssnano from 'cssnano'
import PurgeCSS from '@fullhuman/postcss-purgecss'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const cssDir = resolve(__dirname, '../templates/css')
const outputDir = resolve(__dirname, '../static/dist/css')
const htmlFiles = resolve(__dirname, '../templates/**/*.html')
const jsFiles = resolve(__dirname, '../templates/**/*.js')

// 确保输出目录存在
try {
  mkdirSync(outputDir, { recursive: true })
} catch (err) {
  // 目录已存在
}

// PostCSS 处理器
const processor = postcss([
  // PurgeCSS - 死码消除
  PurgeCSS({
    content: [
      htmlFiles,
      jsFiles
    ],
    safelist: {
      pattern: /^data-/,
      standard: [/^hover:/, /^focus:/, /^active:/, /^visited:/, /^disabled:/],
      keyframes: [
        'fadeIn', 'fadeOut', 'slideIn', 'slideOut', 'bounce',
        'pulse', 'spin', 'shake', 'zoomIn', 'zoomOut', 'flip', 'rotate'
      ],
      greedy: [
        /^modal-/, /^toast-/, /^shortcut-/, /^shortcut-hint/,
        /^article-/, /^sidebar-/, /^file-/, /^filter-/,
        /^tab-/, /^comment-/, /^sponsor-/, /^attachment-/,
        /^preview-/, /^fm-/, /^music-/, /^active$/, /^hidden$/, /^show$/
      ]
    },
    defaultExtractor: (content) => {
      return content.match(/[\w-/:]+(?<!:)/g) || []
    }
  }),
  // Autoprefixer - 自动添加浏览器前缀
  autoprefixer({
    overrideBrowserslist: [
      'last 2 versions',
      'not dead',
      'not IE 11'
    ]
  }),
  // CSSNano - 压缩 CSS
  cssnano({
    preset: [
      'default',
      {
        discardComments: { removeAll: true },
        normalizeWhitespace: true,
        minifyFontValues: true,
        minifySelectors: true,
        reduceIdents: true,
        reduceInitial: true,
        mergeIdents: true,
        mergeRules: true,
        mergeLonghand: true,
        shortHandLongHand: true,
        minifyGradients: true
      }
    ]
  })
])

// 获取所有 CSS 文件
function getCssFiles(dir) {
  const files = []
  const items = readdirSync(dir)

  for (const item of items) {
    const fullPath = join(dir, item)
    const stat = statSync(fullPath)

    if (stat.isDirectory() && item !== 'fonts') {
      files.push(...getCssFiles(fullPath))
    } else if (item.endsWith('.css')) {
      files.push(fullPath)
    }
  }

  return files
}

// 检查文件是否包含模板语法
function containsTemplateSyntax(css) {
  return /{{\s*\w+\.?\w+\s*}}/.test(css)
}

// 保护模板语法：用占位符替换
function protectTemplateSyntax(css) {
  const placeholders = []
  const protectedCss = css.replace(/{{[^}]+}}/g, (match) => {
    const placeholder = `__TEMPLATE_PLACEHOLDER_${placeholders.length}__`
    placeholders.push(match)
    return placeholder
  })
  return { protectedCss, placeholders }
}

// 还原模板语法：将占位符替换回原始模板语法
function restoreTemplateSyntax(css, placeholders) {
  return css.replace(/__TEMPLATE_PLACEHOLDER_(\d+)__/g, (match, index) => {
    return placeholders[parseInt(index)] || match
  })
}

// 处理 CSS 文件
async function optimizeCss(inputPath, outputPath) {
  try {
    // 读取 CSS 文件
    const css = readFileSync(inputPath, 'utf-8')

    let processedCss = css
    const hasTemplateSyntax = containsTemplateSyntax(css)

    // 如果包含模板语法，先保护它们
    let placeholders = []
    if (hasTemplateSyntax) {
      const result = protectTemplateSyntax(css)
      processedCss = result.protectedCss
      placeholders = result.placeholders
    }

    // 处理 CSS
    const result = await processor.process(processedCss, {
      from: inputPath,
      to: outputPath,
      map: false
    })

    // 还原模板语法
    let finalCss = result.css
    if (hasTemplateSyntax) {
      finalCss = restoreTemplateSyntax(result.css, placeholders)
    }

    // 生成文件名（添加 hash）
    const originalName = inputPath.split('/').pop().replace('.css', '')
    const hash = Math.random().toString(36).substring(2, 10)
    const outputFileName = `${originalName}-${hash}.css`
    const outputFilePath = join(outputDir, outputFileName)

    // 写入优化后的 CSS
    writeFileSync(outputFilePath, finalCss)

    // 计算压缩率
    const originalSize = Buffer.byteLength(css, 'utf-8')
    const optimizedSize = Buffer.byteLength(finalCss, 'utf-8')
    const reduction = ((1 - optimizedSize / originalSize) * 100).toFixed(2)

    const templateNotice = hasTemplateSyntax ? ' (包含模板语法)' : ''
    console.log(`✓ ${originalName}.css${templateNotice}`)
    console.log(`  原始大小: ${(originalSize / 1024).toFixed(2)} KB`)
    console.log(`  优化后: ${(optimizedSize / 1024).toFixed(2)} KB`)
    console.log(`  压缩率: ${reduction}%`)
    if (hasTemplateSyntax) {
      console.log(`  受保护的模板语法: ${placeholders.length} 个`)
    }
    console.log()

    return {
      original: originalSize,
      optimized: optimizedSize,
      outputFileName
    }
  } catch (error) {
    console.error(`✗ Error processing ${inputPath}:`, error.message)
    return null
  }
}

// 主函数
async function main() {
  console.log('🎨 CSS 优化开始...\n')

  const cssFiles = getCssFiles(cssDir)
  console.log(`找到 ${cssFiles.length} 个 CSS 文件\n`)

  let totalOriginal = 0
  let totalOptimized = 0
  const outputFiles = []

  // 处理每个 CSS 文件
  for (const file of cssFiles) {
    const result = await optimizeCss(file, outputDir)
    if (result) {
      totalOriginal += result.original
      totalOptimized += result.optimized
      outputFiles.push(result.outputFileName)
    }
  }

  // 输出统计信息
  console.log('📊 统计信息:')
  console.log(`  原始大小: ${(totalOriginal / 1024).toFixed(2)} KB`)
  console.log(`  优化后: ${(totalOptimized / 1024).toFixed(2)} KB`)
  console.log(`  总压缩率: ${((1 - totalOptimized / totalOriginal) * 100).toFixed(2)}%`)
  console.log(`  节省: ${((totalOriginal - totalOptimized) / 1024).toFixed(2)} KB`)
  console.log('\n✅ 优化完成!')

  // 输出文件映射（用于 Rust 后端）
  console.log('\n📝 文件映射（用于 Rust 后端）:')
  const mapping = {}
  for (const file of cssFiles) {
    const originalName = file.split('/').pop()
    const result = outputFiles.find(f => f.startsWith(originalName.replace('.css', '')))
    if (result) {
      mapping[originalName] = result
    }
  }
  console.log(JSON.stringify(mapping, null, 2))
}

// 运行
main().catch(console.error)