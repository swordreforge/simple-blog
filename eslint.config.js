import js from '@eslint/js'

export default [
  js.configs.recommended,
  {
    files: ['templates/**/*.js', 'scripts/**/*.js'],
    languageOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: {
        // 浏览器全局变量
        window: 'readonly',
        document: 'readonly',
        navigator: 'readonly',
        console: 'readonly',
        alert: 'readonly',
        confirm: 'readonly',
        prompt: 'readonly',
        setTimeout: 'readonly',
        setInterval: 'readonly',
        clearTimeout: 'readonly',
        clearInterval: 'readonly',
        fetch: 'readonly',
        XMLHttpRequest: 'readonly',
        FormData: 'readonly',
        URLSearchParams: 'readonly',
        localStorage: 'readonly',
        sessionStorage: 'readonly',
        WebSocket: 'readonly',
        Event: 'readonly',
        EventTarget: 'readonly',
        HTMLElement: 'readonly',
        Element: 'readonly',
        Node: 'readonly',
        DOMException: 'readonly',
        // 编码解码
        atob: 'readonly',
        btoa: 'readonly',
        // 文件 API
        FileReader: 'readonly',
        Blob: 'readonly',
        File: 'readonly',
        // 地理位置
        Geolocation: 'readonly',
        // 其他
        Chart: 'readonly',
        // Node.js 全局变量（用于脚本文件）
        process: 'readonly',
        Buffer: 'readonly',
        __dirname: 'readonly',
        __filename: 'readonly',
        require: 'readonly',
        module: 'readonly',
        exports: 'readonly',
        global: 'readonly'
      }
    },
    rules: {
      // 核心错误检测（未使用的变量、未定义的变量等）
      'no-unused-vars': ['warn', {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_',
        caughtErrorsIgnorePattern: '^_'
      }],
      'no-undef': 'warn',
      'no-redeclare': 'warn',
      'no-dupe-keys': 'warn',
      'no-duplicate-case': 'warn',

      // 严重的语法错误
      'no-extra-semi': 'warn',
      'no-unreachable': 'warn',
      'valid-typeof': 'warn',
      'no-const-assign': 'warn',
      'no-new-native-nonconstructor': 'warn',
      'no-obj-calls': 'warn',
      'no-self-assign': 'warn',
      'no-sparse-arrays': 'warn',
      'use-isnan': 'warn',

      // 代码质量警告（但不是错误）
      'no-console': 'off',
      'no-debugger': 'off',
      'no-alert': 'off',
      'no-var': 'off',
      'prefer-const': 'off',
      'no-prototype-builtins': 'off',
      'no-return-await': 'off',
      'require-atomic-updates': 'off',
      'eqeqeq': 'off',
      'no-eval': 'off',
      'no-implied-eval': 'off',
      'no-new-func': 'off',
      'no-script-url': 'off',
      'no-with': 'off',
      'no-delete-var': 'off',
      'no-throw-literal': 'off',
      'curly': 'off',
      'default-case': 'off',
      'no-else-return': 'off',
      'no-empty-function': 'off',
      'no-multi-spaces': 'off',
      'no-multiple-empty-lines': 'off',
      'no-trailing-spaces': 'off',
      'no-unused-expressions': 'off',
      'no-useless-return': 'off',
      'yoda': 'off',
      'arrow-body-style': 'off',
      'arrow-parens': 'off',
      'arrow-spacing': 'off',
      'no-duplicate-imports': 'off',
      'no-useless-constructor': 'off',
      'object-shorthand': 'off',
      'prefer-arrow-callback': 'off',
      'prefer-rest-params': 'off',
      'prefer-spread': 'off',
      'prefer-template': 'off',
      'rest-spread-spacing': 'off',
      'no-empty': 'off',
      'no-irregular-whitespace': 'off',
      'no-unsafe-finally': 'off',
      'no-unsafe-negation': 'off',
      'no-useless-escape': 'off'
    }
  },
  {
    files: ['scripts/**/*.js'],
    rules: {
      // 脚本文件允许未定义的变量（可能通过 require 引入）
      'no-undef': 'off'
    }
  },
  {
    ignores: [
      'node_modules/**',
      'dist/**',
      'static/dist/**',
      'target/**',
      '*.min.js',
      'templates/js/*.min.js'
    ]
  }
]