import { readFileSync, writeFileSync, readdirSync } from 'fs'
import { join } from 'path'

const templatesDir = 'templates'
const htmlFiles = readdirSync(templatesDir).filter(f => f.endsWith('.html'))

htmlFiles.forEach(file => {
  const filePath = join(templatesDir, file)
  console.log(`Processing ${file}...`)

  let content = readFileSync(filePath, 'utf-8')

  // 替换所有 <script src="/js/..."> 为 <script type="module" src="/js/...">
  content = content.replace(
    /<script\s+src="(\/js\/[^\"]+)"\s*><\/script>/g,
    '<script type="module" src="$1"></script>'
  )

  // 替换 <script src="/js/...?v=..."> 为带 module 的版本
  content = content.replace(
    /<script\s+src="(\/js\/[^\"]+\?v=\d+)"\s*><\/script>/g,
    '<script type="module" src="$1"></script>'
  )

  writeFileSync(filePath, content, 'utf-8')
  console.log(`✓ ${file} processed`)
})

console.log('\nAll HTML files have been processed!')