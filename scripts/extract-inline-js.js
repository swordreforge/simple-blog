import { readFileSync, writeFileSync, readdirSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

/**
 * 提取 HTML 中的行内脚本到外部文件
 * 自动处理模板变量（如 {{ variable }}）
 */
class InlineJSExtractor {
  constructor(templatesDir = 'templates', outputDir = 'templates/js') {
    this.templatesDir = templatesDir
    this.outputDir = outputDir
    this.templateVars = new Set()
  }

  /**
   * 查找所有包含行内脚本的 HTML 文件
   */
  findHTMLFiles() {
    const files = readdirSync(this.templatesDir, { recursive: true })
    return files.filter(f => f.endsWith('.html'))
  }

  /**
   * 检测模板变量并标记
   * 只检测在 JavaScript 代码上下文中的 Askama 模板变量 {{ }}
   * 忽略 JavaScript 模板字符串 ${}
   */
  detectTemplateVars(content) {
    // 先移除 JavaScript 模板字符串，避免误判
    // 匹配 ${...} 并替换为空字符串
    const withoutTemplateLiterals = content.replace(/\$\{[^}]*\}/g, '')

    // 清理内容，移除注释以避免误判
    const cleanedContent = withoutTemplateLiterals
      .replace(/\/\/.*$/gm, '')  // 单行注释
      .replace(/\/\*[\s\S]*?\*\//g, '')  // 多行注释
      .replace(/<!--[\s\S]*?-->/g, '')  // HTML 注释

    // 检测 Askama 模板变量 {{ variable }}
    const patterns = [
      // Askama 变量语法: {{ variable }}
      /\{\{\s*[\w.]+\s*\}\}/g,
      // Askama 控制结构: {% if %}, {% for %}, etc.
      /\{%\s*(?:if|for|else|endif|endfor|include|extends|let|macro)\s*.*?%\}/g,
      // Askama 注释: {# comment #}
      /\{\#.*?\#\}/g,
    ]

    patterns.forEach(pattern => {
      const matches = cleanedContent.match(pattern)
      if (matches) {
        matches.forEach(m => this.templateVars.add(m))
      }
    })

    return this.templateVars.size > 0
  }

  /**
   * 将模板变量转换为 JavaScript 可用的形式
   * {{ variable }} -> window.templateVariable = "{{ variable }}"
   */
  convertTemplateVarsToJS(content) {
    // 保留模板变量原样，它们会在服务端渲染时替换
    return content
  }

  /**
   * 提取单个 HTML 文件中的行内脚本
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

      // 跳过空脚本
      if (!scriptContent) continue

      // 检查是否包含模板变量
      const hasTemplateVars = this.detectTemplateVars(scriptContent)

      matches.push({
        fullScript,
        scriptContent,
        hasTemplateVars,
        startIndex: match.index,
        endIndex: match.index + fullScript.length
      })
    }

    if (matches.length === 0) {
      console.log(`  ✓ 没有行内脚本`)
      return { extracted: false }
    }

    // 生成外部 JS 文件
    const baseName = filePath.replace(/\.html$/, '')
    const scriptFiles = []

    matches.forEach((scriptData, index) => {
      const scriptFileName = `${baseName}-inline-${index + 1}.js`
      const scriptFilePath = join(this.outputDir, scriptFileName)

      // 处理脚本内容
      let processedContent = scriptData.scriptContent

      // 如果包含模板变量，保持原样（Askama 会处理）
      // 否则可以安全地提取到外部文件
      if (scriptData.hasTemplateVars) {
        console.log(`  ⚠ 脚本 ${index + 1} 包含模板变量，保持行内`)
        return
      }

      // 写入外部文件
      writeFileSync(scriptFilePath, processedContent, 'utf-8')
      scriptFiles.push(scriptFileName)
      console.log(`  ✓ 提取脚本 ${index + 1} -> ${scriptFileName}`)
    })

    if (scriptFiles.length === 0) {
      console.log(`  ⚠ 所有脚本都包含模板变量，无法提取`)
      return { extracted: false }
    }

    // 更新 HTML 文件，替换行内脚本为外部引用
    let newContent = content
    let offset = 0

    matches.forEach((scriptData, index) => {
      if (!scriptData.hasTemplateVars) {
        const scriptFileName = scriptFiles.shift()
        const externalScript = `<script type="module" src="/js/${scriptFileName}"></script>`

        newContent = newContent.substring(0, scriptData.startIndex + offset) +
                     externalScript +
                     newContent.substring(scriptData.endIndex + offset)

        offset += externalScript.length - scriptData.fullScript.length
      }
    })

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
   * 批量提取所有 HTML 文件
   */
  extractAll() {
    console.log('🔍 扫描 HTML 文件...\n')
    const htmlFiles = this.findHTMLFiles()

    if (htmlFiles.length === 0) {
      console.log('❌ 没有找到 HTML 文件')
      return
    }

    console.log(`找到 ${htmlFiles.length} 个 HTML 文件\n`)

    const results = {
      total: htmlFiles.length,
      extracted: 0,
      skipped: 0,
      details: []
    }

    htmlFiles.forEach(file => {
      console.log(`\n📄 处理: ${file}`)
      const result = this.extractFromFile(file)

      if (result.extracted) {
        results.extracted++
        results.details.push({
          file,
          scriptFiles: result.scriptFiles,
          backupPath: result.backupPath
        })
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
  console.log('🚀 行内 JS 提取工具\n')

  const extractor = new InlineJSExtractor('templates', 'templates/js')
  extractor.extractAll()
}

// 运行
if (import.meta.url === `file://${process.argv[1]}`) {
  main()
}

export { InlineJSExtractor }