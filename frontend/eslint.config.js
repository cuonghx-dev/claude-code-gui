import pluginVue from 'eslint-plugin-vue'
import tseslint from 'typescript-eslint'

export default [
  // Without a TypeScript parser, eslint choked on the first `as` or type
  // annotation in every file — the whole lint run was parse errors, so it
  // proved nothing.
  ...tseslint.configs.recommended,
  ...pluginVue.configs['flat/recommended'],
  {
    files: ['**/*.vue'],
    languageOptions: {
      // vue-eslint-parser handles the SFC; the script block goes to
      // typescript-eslint.
      parserOptions: { parser: tseslint.parser, ecmaVersion: 'latest', sourceType: 'module' },
    },
  },
  {
    files: ['**/*.{js,ts,vue}'],
    rules: {
      'vue/multi-word-component-names': 'off',
      // Pure formatting, and the codebase does not follow them. With no
      // formatter in the repo they would be 900 warnings of noise that hide
      // the findings worth reading.
      'vue/max-attributes-per-line': 'off',
      'vue/singleline-html-element-content-newline': 'off',
      'vue/html-self-closing': 'off',
      'vue/attributes-order': 'off',
      // Generated bindings and Tauri payloads are structurally typed at the
      // boundary; an unused arg prefixed with _ is deliberate.
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
    },
  },
  {
    ignores: ['dist/**', 'node_modules/**', 'src/types/ipc/**', 'src/typed-router.d.ts'],
  },
]
