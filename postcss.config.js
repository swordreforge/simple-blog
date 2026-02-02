export default {
  plugins: {
    // 自动添加浏览器前缀
    autoprefixer: {
      overrideBrowserslist: [
        'last 2 versions',
        'not dead',
        'not IE 11'
      ]
    },
    // CSS 压缩
    cssnano: {
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
    }
  }
}