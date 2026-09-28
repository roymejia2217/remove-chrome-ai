module.exports = {
  extends: ['@commitlint/config-conventional'],
  defaultIgnores: false,
  rules: {
    'type-enum': [
      2,
      'always',
      [
        'build',
        'chore',
        'ci',
        'docs',
        'feat',
        'fix',
        'perf',
        'refactor',
        'revert',
        'style',
        'test'
      ]
    ],
    'scope-enum': [
      2,
      'always',
      [
        'core',
        'cli',
        'config',
        'scanner',
        'cleaner',
        'policy',
        'reporting',
        'ci',
        'deps',
        'docs',
        'governance',
        'release'
      ]
    ],
    'header-max-length': [2, 'always', 72],
    'body-empty': [2, 'never'],
    'body-min-length': [2, 'always', 20],
    'body-leading-blank': [2, 'always'],
    'body-max-line-length': [2, 'always', 100],
    'footer-leading-blank': [2, 'always'],
    'footer-max-line-length': [2, 'always', 100]
  }
};
