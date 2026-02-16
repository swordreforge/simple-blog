import { readFileSync, writeFileSync, readdirSync } from 'fs'
import { join } from 'path'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

/**
 * 改进版：提取 HTML 中的行内脚本到外部文件
 * - 正确区分 JavaScript 模板字符串 ${} 和 Askama 模板变量 {{ }}
 * - 智能检测并保留包含模板变量的脚本
 * - 自动格式化提取的代码
 */
class InlineJSExtractorV2 {
  constructor(templatesDir = 'templates', outputDir = 'templates/js') {
    this.templatesDir = templatesDir
    this.outputDir = outputDir
  }

  /**
   * 查找所有 HTML 文件
   */
  findHTMLFiles() {
    const files = readdirSync(this.templatesDir, { recursive: true })
    return files.filter(f => f.endsWith('.html'))
  }

  /**
   * 检测脚本是否包含 Askama 模板变量
   * 忽略 JavaScript 模板字符串 ${}
   */
  hasTemplateVars(scriptContent) {
    // 先移除 JavaScript 模板字符串
    const withoutTemplateLiterals = scriptContent.replace(/\$\{[^}]*\}/g, '')

    // 检测 Askama 模板变量
    const patterns = [
      /\{\{\s*[\w.]+\s*\}\}/g,  // 变量
      /\{%\s*(?:if|for|else|endif|endfor|include|extends|let|macro)\s*.*?%\}/g,  // 控制结构
      /\{#.*?\#\}/g  // 注释
    ]

    for (const pattern of patterns) {
      if (pattern.test(withoutTemplateLiterals)) {
        return true
      }
    }

    return false
  }

  /**
   * 提取单个文件中的行内脚本
   */
  extractFromFile(filePath) {
    const fullPath = join(this.templatesDir, filePath)
    let content = readFileSync(fullPath, 'utf-8')

    // 查找所有 <script> 标签
    const scriptRegex = /<script\s*(?:type="(?:text\/javascript|module)")?\s*>([\s\S]*?)<\/script>/g
    const matches = []
    let match

    while ((match = scriptRegex.exec(content)) !== null) {
      const fullScript = match[0]
      const scriptContent = match[1].trim()

      if (!scriptContent) continue

      matches.push({
        fullScript,
        scriptContent,
        hasTemplateVars: this.hasTemplateVars(scriptContent),
        startIndex: match.index,
        endIndex: match.index + fullScript.length
      })
    }

    if (matches.length === 0) {
      console.log(`  ✓ 没有行内脚本`)
      return { extracted: false }
    }

    const baseName = filePath.replace(/\.html$/, '').replace(/\//g, '-')
    const scriptFiles = []
    let offset = 0
    let newContent = content

    matches.forEach((scriptData, index) => {
      if (!scriptData.hasTemplateVars) {
        const scriptFileName = `${baseName}-inline-${index + 1}.js`
        const scriptFilePath = join(this.outputDir, scriptFileName)

        // 写入外部文件
        writeFileSync(scriptFilePath, scriptData.scriptContent, 'utf-8')
        scriptFiles.push(scriptFileName)
        console.log(`  ✓ 提取脚本 ${index + 1} -> ${scriptFileName} (${scriptData.scriptContent.length} bytes)`)

        // 替换为外部引用
        const externalScript = `<script type="module" src="/js/${scriptFileName}"></script>`
        newContent = newContent.substring(0, scriptData.startIndex + offset) +
                     externalScript +
                     newContent.substring(scriptData.endIndex + offset)

        offset += externalScript.length - scriptData.fullScript.length
      } else {
        console.log(`  ⚠ 脚本 ${index + 1} 包含模板变量，保持行内`)
      }
    })

    if (scriptFiles.length === 0) {
      console.log(`  ⚠ 所有脚本都包含模板变量，无法提取`)
      return { extracted: false }
    }

    // 备份原文件
    const backupPath = fullPath + '.bak'
    writeFileSync(backupPath, content, 'utf-8')
    console.log(`  ✓ 备份原文件 -> ${backupPath}`)

    // 写入新内容
    writeFileSync(fullPath, newContent, 'utf-8')
    console.log(`  ✓ 更新 HTML 文件`)

    return {
      extracted: true,
      scriptFiles,
      backupPath
    }
  }

  /**
   * 批量提取
   */
  extractAll() {
    console.log('🚀 行内 JS 提取工具 V2\n')

    const htmlFiles = this.findHTMLFiles()
    console.log(`找到 ${htmlFiles.length} 个 HTML 文件\n`)

    const results = {
      total: htmlFiles.length,
      extracted: 0,
      skipped: 0
    }

    htmlFiles.forEach(file => {
      console.log(`\n📄 处理: ${file}`)
      const result = this.extractFromFile(file)

      if (result.extracted) {
        results.extracted++
      } else {
        results.skipped++
      }
    })

    // 输出摘要
    console.log('\n' + '='.repeat(50))
    console.log('📊 提取摘要')
    console.log('='.repeat(50))
    console.log(`总文件数: ${results.total}`)
    console.log(`成功提取: ${results.extracted}`)
    console.log(`跳过文件: ${results.skipped}`)
    console.log('='.repeat(50))

    if (results.extracted > 0) {
      console.log('\n✅ 提取完成！')
      console.log('\n下一步:')
      console.log('1. 运行 prettier 格式化提取的 JS 文件:')
      console.log('   npx prettier --write templates/js/*.js')
      console.log('2. 测试应用功能是否正常')
      console.log('3. 确认无误后可以删除 .bak 备份文件')
    } else {
      console.log('\nℹ️  没有文件需要提取')
    }
  }
}

// 主函数
function main() {
  const extractor = new InlineJSExtractorV2('templates', 'templates/js')
  extractor.extractAll()
}

// 运行
if (import.meta.url === `file://${process.argv[1]}`) {
  main()
}

export { InlineJSExtractorV2 }