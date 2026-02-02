import { defineConfig } from 'vite'
import { resolve } from 'path'

export default defineConfig({
  // 根目录
  root: '.',

  // 构建配置
  build: {
    // 输出目录
    outDir: 'static/dist',
    // 清空输出目录
    emptyOutDir: true,
    // 生成 sourcemap
    sourcemap: false,
    // 压缩配置
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: false,
        drop_debugger: true,
        pure_funcs: ['console.log']
      },
      format: {
        comments: false
      }
    },
    // Rollup 配置
    rollupOptions: {
      input: {
        // JS 入口文件
        index: resolve(__dirname, 'templates/js/index.js'),
        passage: resolve(__dirname, 'templates/js/passage.js'),
        // 单独打包其他 JS 文件
        'keyboard-shortcuts': resolve(__dirname, 'templates/js/keyboard-shortcuts.js'),
        'modal-animations': resolve(__dirname, 'templates/js/modal-animations.js'),
        'passage-shortcuts': resolve(__dirname, 'templates/js/passage-shortcuts.js'),
        'markdown-preview-modal': resolve(__dirname, 'templates/js/markdown-preview-modal.js'),
        'passage-focus-mode': resolve(__dirname, 'templates/js/passage-focus-mode.js'),
        'collect-focus-mode': resolve(__dirname, 'templates/js/collect-focus-mode.js'),
        'about-focus-mode': resolve(__dirname, 'templates/js/about-focus-mode.js'),
        'music-player': resolve(__dirname, 'templates/js/music-player.js'),
        'filemanager': resolve(__dirname, 'templates/js/filemanager.js'),
        'login': resolve(__dirname, 'templates/js/login.js'),
        'ecc-encrypt': resolve(__dirname, 'templates/js/ecc-encrypt.js'),
        'floating-text': resolve(__dirname, 'templates/js/floating-text.js'),
        'chart': resolve(__dirname, 'templates/js/chart.js'),
      },
      output: {
        // 输出文件命名
        entryFileNames: 'js/[name]-[hash].js',
        chunkFileNames: 'js/[name]-[hash].js',
        assetFileNames: (assetInfo) => {
          const info = assetInfo.name.split('.')
          const ext = info[info.length - 1]
          if (/\.(css)$/.test(assetInfo.name)) {
            return `css/[name]-[hash].${ext}`
          }
          if (/\.(png|jpe?g|gif|svg|webp|ico)$/.test(assetInfo.name)) {
            return `img/[name]-[hash].${ext}`
          }
          if (/\.(woff2?|eot|ttf|otf)$/.test(assetInfo.name)) {
            return `fonts/[name]-[hash].${ext}`
          }
          return `assets/[name]-[hash].${ext}`
        }
      }
    }
  },

  // 路径别名
  resolve: {
    alias: {
      '@': resolve(__dirname, 'templates'),
      '@css': resolve(__dirname, 'templates/css'),
      '@js': resolve(__dirname, 'templates/js')
    }
  },

  // 资源处理
  assetsInclude: ['**/*.webp', '**/*.svg'],
})