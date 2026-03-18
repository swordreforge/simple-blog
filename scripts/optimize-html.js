import { readFileSync, writeFileSync, readdirSync, mkdirSync, statSync } from 'fs'
import { resolve, dirname, join } from 'path'
import { fileURLToPath } from 'url'
import { minify } from 'html-minifier-terser'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const templatesDir = resolve(__dirname, '../templates')
const outputDir = resolve(__dirname, '../static/dist/templates')

// 确保输出目录存在
try {
  mkdirSync(outputDir, { recursive: true })
} catch (err) {
  // 目录已存在
}

// 获取所有 HTML 文件
function getHtmlFiles(dir, excludeDirs = ['node_modules', 'dist', 'static']) {
  const files = []
  const items = readdirSync(dir)

  for (const item of items) {
    const fullPath = join(dir, item)
    const stat = statSync(fullPath)

    if (stat.isDirectory() && !excludeDirs.includes(item)) {
      files.push(...getHtmlFiles(fullPath, excludeDirs))
    } else if (item.endsWith('.html') && !item.includes('.dev.html')) {
      files.push(fullPath)
    }
  }

  return files
}

// 处理 HTML 文件
async function optimizeHtml(inputPath, outputPath) {
  try {
    // 读取 HTML 文件
    const html = readFileSync(inputPath, 'utf-8')

    // 压缩 HTML
    const minified = await minify(html, {
      // 移除注释
      removeComments: true,
      // 移除空属性
      removeEmptyAttributes: true,
      // 移除可省略的标签
      removeOptionalTags: true,
      // 移除冗余属性
      removeRedundantAttributes: true,
      // 移除 script 的 type 属性
      removeScriptTypeAttributes: true,
      // 移除 style 的 type 属性
      removeStyleLinkTypeAttributes: true,
      // 移除空格
      collapseWhitespace: true,
      // 保留一个空格
      collapseBooleanAttributes: true,
      // 删除额外的引号
      removeAttributeQuotes: false, // 保留引号以确保兼容性
      // 压缩内联 CSS
      minifyCSS: true,
      // 压缩内联 JS
      minifyJS: true,
      // 忽略自定义片段
      ignoreCustomComments: [/^\s*{{/],
      // 保留 ES6 语法
      ignoreCustomFragments: [
        /{{[\s\S]*?}}/,
        /{%[\s\S]*?%}/,
        /{#[\s\S]*?#}/
      ],
      // 保持模板语法
      caseSensitive: true,
      // 移除换行
      removeEmptyElements: false,
      // 合并多个空格
      keepClosingSlash: false,
      // 防止破坏模板语法
      continueOnParseError: false
    })

    // 如果压缩失败，返回原始内容
    const output = minified || html

    // 计算压缩率
    const originalSize = Buffer.byteLength(html, 'utf-8')
    const optimizedSize = Buffer.byteLength(output, 'utf-8')
    const reduction = ((1 - optimizedSize / originalSize) * 100).toFixed(2)

    // 保持相对路径结构
    const relativePath = inputPath.replace(templatesDir, '')
    const outputFile = join(outputDir, relativePath)
    const outputFileDir = dirname(outputFile)

    // 确保子目录存在
    try {
      mkdirSync(outputFileDir, { recursive: true })
    } catch (err) {
      // 目录已存在
    }

    // 写入优化后的 HTML
    writeFileSync(outputFile, output)

    const fileName = inputPath.split('/').pop()
    console.log(`✓ ${fileName}`)
    console.log(`  原始大小: ${(originalSize / 1024).toFixed(2)} KB`)
    console.log(`  优化后: ${(optimizedSize / 1024).toFixed(2)} KB`)
    console.log(`  压缩率: ${reduction}%`)
    console.log()

    return {
      original: originalSize,
      optimized: optimizedSize,
      fileName
    }
  } catch (error) {
    console.error(`✗ Error processing ${inputPath}:`, error.message)
    return null
  }
}

// 主函数
async function main() {
  console.log('📄 HTML 优化开始...\n')

  // 获取命令行参数
  const args = process.argv.slice(2)
  let htmlFiles = []

  if (args.length > 0) {
    // 单文件模式：处理指定的文件
    const targetFile = resolve(args[0])

    if (!targetFile.endsWith('.html')) {
      console.error('✗ 错误：只支持 .html 文件')
      process.exit(1)
    }

    if (targetFile.startsWith(templatesDir)) {
      htmlFiles = [targetFile]
      console.log(`单文件模式: ${args[0]}\n`)
    } else {
      console.error('✗ 错误：文件必须在 templates 目录下')
      process.exit(1)
    }
  } else {
    // 批量模式：处理所有 HTML 文件
    htmlFiles = getHtmlFiles(templatesDir)
    console.log(`批量模式：找到 ${htmlFiles.length} 个 HTML 文件\n`)
  }

  let totalOriginal = 0
  let totalOptimized = 0

  // 处理每个 HTML 文件
  for (const file of htmlFiles) {
    const result = await optimizeHtml(file, outputDir)
    if (result) {
      totalOriginal += result.original
      totalOptimized += result.optimized
    }
  }

  // 输出统计信息
  console.log('📊 统计信息:')
  console.log(`  原始大小: ${(totalOriginal / 1024).toFixed(2)} KB`)
  console.log(`  优化后: ${(totalOptimized / 1024).toFixed(2)} KB`)
  console.log(`  总压缩率: ${((1 - totalOptimized / totalOriginal) * 100).toFixed(2)}%`)
  console.log(`  节省: ${((totalOriginal - totalOptimized) / 1024).toFixed(2)} KB`)
  console.log('\n✅ 优化完成!')
}

// 运行
main().catch(console.error)