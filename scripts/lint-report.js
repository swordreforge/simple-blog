import { execSync } from 'child_process'
import { writeFileSync } from 'fs'

console.log('🔍 运行 ESLint 检查...\n')

try {
  // 运行 ESLint 并捕获输出
  let output = ''
  try {
    output = execSync('npm run lint 2>&1', { encoding: 'utf-8' })
  } catch (e) {
    // ESLint 返回非零退出码是正常的，但我们需要捕获输出
    output = e.stdout || e.stderr || ''
  }

  // 解析输出
  const lines = output.split('\n')
  const issues = []

  for (const line of lines) {
    if (line.includes('no-unused-vars') || line.includes('no-undef')) {
      issues.push(line)
    }
  }

  // 统计各类问题
  const unusedVars = issues.filter(line => line.includes('no-unused-vars'))
  const undefVars = issues.filter(line => line.includes('no-undef'))

  // 按文件分组
  const fileStats = {}
  for (const issue of issues) {
    const match = issue.match(/\/([^\/]+\.js)/)
    if (match) {
      const fileName = match[1]
      if (!fileStats[fileName]) {
        fileStats[fileName] = 0
      }
      fileStats[fileName]++
    }
  }

  // 生成报告
  console.log('📊 ESLint 检查报告\n')
  console.log('========================================')
  console.log(`未使用的变量 (no-unused-vars): ${unusedVars.length}`)
  console.log(`未定义的变量 (no-undef): ${undefVars.length}`)
  console.log(`总计: ${issues.length} 个问题`)
  console.log('========================================\n')

  console.log('📁 按文件统计:\n')
  const sortedFiles = Object.entries(fileStats).sort((a, b) => b[1] - a[1])
  for (const [fileName, count] of sortedFiles.slice(0, 10)) {
    console.log(`  ${fileName}: ${count} 个问题`)
  }

  console.log('\n💡 建议操作:\n')
  console.log('  1. 自动修复可以修复的问题:')
  console.log('     npm run lint:fix\n')
  console.log('  2. 手动修复未定义的变量（添加到全局变量或导入）')
  console.log('  3. 手动修复未使用的变量（删除或添加下划线前缀）\n')

  // 保存报告到文件
  const report = {
    summary: {
      unusedVars: unusedVars.length,
      undefVars: undefVars.length,
      total: issues.length
    },
    fileStats: sortedFiles,
    issues: issues
  }

  writeFileSync('eslint-report.json', JSON.stringify(report, null, 2))
  console.log('✅ 详细报告已保存到 eslint-report.json')

} catch (error) {
  console.error('❌ 运行 ESLint 时出错:', error.message)
  process.exit(1)
}