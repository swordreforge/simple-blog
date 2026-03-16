#!/usr/bin/env node
/**
 * 压缩 HTML 模板文件为单行
 * 直接替换原始文件，并保留备份
 */

import { readFileSync, writeFileSync, readdirSync, mkdirSync, statSync, copyFileSync, existsSync } from 'fs'
import { resolve, dirname, join } from 'path'
import { fileURLToPath } from 'url'
import { minify } from 'html-minifier-terser'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const templatesDir = resolve(__dirname, '../templates')
const backupDir = resolve(__dirname, '../backups/original-html')

// 创建备份目录
if (!existsSync(backupDir)) {
  mkdirSync(backupDir, { recursive: true })
  console.log(`创建备份目录: ${backupDir}`)
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
async function compressHtml(filePath) {
  try {
    // 读取 HTML 文件
    const html = readFileSync(filePath, 'utf-8')

    // 备份原始文件
    const relativePath = filePath.replace(templatesDir, '')
    const backupPath = join(backupDir, relativePath)
    const backupDirPath = dirname(backupPath)
    
    try {
      mkdirSync(backupDirPath, { recursive: true })
    } catch (err) {
      // 目录已存在
    }
    
    copyFileSync(filePath, backupPath)

    // 压缩 HTML
    const minified = await minify(html, {
      // 移除注释（保留模板注释）
      removeComments: true,
      // 移除空属性
      removeEmptyAttributes: true,
      // 移除可省略的标签
      removeOptionalTags: false, // 保留标签以确保兼容性
      // 移除冗余属性
      removeRedundantAttributes: true,
      // 移除 script 的 type 属性
      removeScriptTypeAttributes: true,
      // 移除 style 的 type 属性
      removeStyleLinkTypeAttributes: true,
      // 移除空格并压缩为单行
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
      // 移除空元素
      removeEmptyElements: false,
      // 不保留斜杠
      keepClosingSlash: false,
      // 防止破坏模板语法
      continueOnParseError: false,
      // 移除换行
      minifyURLs: false
    })

    // 如果压缩失败，返回原始内容
    const output = minified || html

    // 计算压缩率
    const originalSize = Buffer.byteLength(html, 'utf-8')
    const compressedSize = Buffer.byteLength(output, 'utf-8')
    const reduction = ((1 - compressedSize / originalSize) * 100).toFixed(2)

    // 写入压缩后的 HTML（覆盖原始文件）
    writeFileSync(filePath, output)

    const fileName = filePath.split('/').pop()
    console.log(`✓ ${fileName}`)
    console.log(`  原始大小: ${(originalSize / 1024).toFixed(2)} KB`)
    console.log(`  压缩后: ${(compressedSize / 1024).toFixed(2)} KB`)
    console.log(`  压缩率: ${reduction}%`)
    console.log(`  备份: ${relativePath}`)
    console.log()

    return {
      original: originalSize,
      compressed: compressedSize,
      fileName
    }
  } catch (error) {
    console.error(`✗ Error processing ${filePath}:`, error.message)
    return null
  }
}

// 主函数
async function main() {
  console.log('📄 HTML 压缩开始...\n')
  console.log(`备份目录: ${backupDir}\n`)

  const htmlFiles = getHtmlFiles(templatesDir)
  console.log(`找到 ${htmlFiles.length} 个 HTML 文件\n`)

  let totalOriginal = 0
  let totalCompressed = 0
  let skippedCount = 0

  // 处理每个 HTML 文件
  for (const file of htmlFiles) {
    const result = await compressHtml(file)
    if (result) {
      if (result.skipped) {
        console.log(`- 跳过: ${result.fileName} (已压缩)\n`)
        skippedCount++
      } else {
        totalOriginal += result.original
        totalCompressed += result.compressed
      }
    }
  }

  // 输出统计信息
  console.log('📊 统计信息:')
  if (htmlFiles.length > skippedCount) {
    console.log(`  原始大小: ${(totalOriginal / 1024).toFixed(2)} KB`)
    console.log(`  压缩后: ${(totalCompressed / 1024).toFixed(2)} KB`)
    console.log(`  总压缩率: ${((1 - totalCompressed / totalOriginal) * 100).toFixed(2)}%`)
    console.log(`  节省: ${((totalOriginal - totalCompressed) / 1024).toFixed(2)} KB`)
  }
  console.log(`  跳过: ${skippedCount} 个文件`)
  console.log(`\n✅ 压缩完成!`)
  console.log(`\n提示: 如需恢复原始文件，请从 backups/original-html/ 目录复制回来`)
}

// 运行
main().catch(console.error)
