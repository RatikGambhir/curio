import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from 'typescript-eslint'
import { globalIgnores } from 'eslint/config'

const tauriImportPattern = {
  group: ['@tauri-apps/*'],
  message:
    'Tauri APIs belong only in src/platform/desktop.ts; shared code must use the platform contract.',
}

const platformRuntimePattern = {
  group: ['@curio/platform-runtime'],
  message:
    'Only src/api and src/features/chat may use the platform runtime; call the typed API layer instead.',
}

export default tseslint.config([
  globalIgnores(['dist', 'src-tauri']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      reactHooks.configs['recommended-latest'],
      reactRefresh.configs.vite,
    ],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [tauriImportPattern, platformRuntimePattern],
        },
      ],
      'no-restricted-globals': [
        'error',
        {
          name: 'fetch',
          message:
            'External requests belong in src/platform/web.ts or in a Rust command behind src/platform/desktop.ts.',
        },
      ],
    },
  },
  {
    files: ['src/api/**/*.{ts,tsx}', 'src/features/chat/**/*.{ts,tsx}'],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [tauriImportPattern],
        },
      ],
    },
  },
  {
    files: ['src/platform/desktop.ts'],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['@tauri-apps/plugin-*'],
              message:
                'Desktop OS integrations must use an app-owned command or event implemented in Rust.',
            },
          ],
        },
      ],
    },
  },
  {
    files: ['src/platform/web.ts'],
    rules: {
      'no-restricted-globals': 'off',
    },
  },
])
