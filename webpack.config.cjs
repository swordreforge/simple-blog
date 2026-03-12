const path = require('path');
const { BundleAnalyzerPlugin } = require('webpack-bundle-analyzer');

module.exports = {
  mode: 'production',
  entry: {
    // 分析所有主要的 JS 文件
    'chart': './templates/js/chart.js',
    'keyboard-shortcuts': './templates/js/keyboard-shortcuts.js',
    'filemanager': './templates/js/filemanager.js',
    'music-player': './templates/js/music-player.js',
    'login': './templates/js/login.js',
    'admin-4730': './templates/js/admin-4730.js',
    'admin-inline': './templates/js/admin-inline.js',
    'admin-inline-1': './templates/js/admin-inline-1.js',
    'admin-inline-2': './templates/js/admin-inline-2.js',
    'admin-inline-4': './templates/js/admin-inline-4.js',
    'about-inline-1': './templates/js/about-inline-1.js',
    'about-inline-2': './templates/js/about-inline-2.js',
    'passage-focus-mode': './templates/js/passage-focus-mode.js',
    'collect-focus-mode': './templates/js/collect-focus-mode.js',
    'floating-text': './templates/js/floating-text.js',
    'ecc-encrypt': './templates/js/ecc-encrypt.js',
  },
  output: {
    path: path.resolve(__dirname, 'static/dist/bundle-analysis'),
    filename: '[name].bundle.js',
    clean: true,
  },
  optimization: {
    minimize: true,
    usedExports: true, // Tree shaking
    sideEffects: false, // 所有文件都有副作用
  },
  plugins: [
    new BundleAnalyzerPlugin({
      analyzerMode: 'static', // 生成静态 HTML 报告
      reportFilename: '../bundle-report.html',
      openAnalyzer: false,
      generateStatsFile: true,
      statsFilename: '../bundle-stats.json',
      statsOptions: {
        source: false,
        modules: true,
        chunks: true,
        chunkModules: true,
      },
    }),
  ],
  performance: {
    maxAssetSize: 500000,
    maxEntrypointSize: 500000,
    hints: 'warning',
  },
};