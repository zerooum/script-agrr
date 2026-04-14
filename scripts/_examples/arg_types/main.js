'use strict';

const path = require('path');
const { createAgrrScript } = require(path.join(__dirname, '..', '..', '..', 'sdk', 'js', 'index.js'));

createAgrrScript({
  meta: {
    name: 'Arg Types Example (JS)',
    description: 'Demonstrates text, select, and multiselect args with all supported constraints',
    group: 'Examples',
    version: '1.0.0',
    args: [
      // text: basic
      { name: 'name', prompt: 'Your name:', type: 'text' },
      // text: max_length + default
      { name: 'short_code', prompt: 'Short code (max 8 chars, default: abc):', type: 'text', max_length: 8, default: 'abc' },
      // text: numeric pattern
      { name: 'age', prompt: 'Age (numbers only, max 3 digits):', type: 'text', pattern: 'numeric', max_length: 3 },
      // text: alpha pattern, optional
      { name: 'suffix', prompt: 'Suffix (letters only, optional):', type: 'text', pattern: 'alpha', required: false },
      // text: alphanumeric pattern
      { name: 'code', prompt: 'Alphanumeric code (max 8 chars):', type: 'text', pattern: 'alphanumeric', max_length: 8 },
      // select: with default
      { name: 'environment', prompt: 'Environment:', type: 'select', options: ['dev', 'staging', 'prod'], default: 'dev' },
      // select: no default
      { name: 'priority', prompt: 'Priority:', type: 'select', options: ['low', 'medium', 'high', 'critical'] },
      // multiselect: required
      { name: 'regions', prompt: 'Deploy regions:', type: 'multiselect', options: ['us-east-1', 'us-west-2', 'eu-west-1', 'ap-southeast-1'] },
      // multiselect: optional with default
      { name: 'channels', prompt: 'Notification channels (optional):', type: 'multiselect', options: ['email', 'slack', 'pagerduty', 'webhook'], default: 'email,slack', required: false },
    ],
  },

  async run({ args }) {
    console.log('=== Arg Types Example ===\n');

    const sections = [
      ['text', ['name', 'short_code', 'age', 'suffix', 'code']],
      ['select', ['environment', 'priority']],
      ['multiselect', ['regions', 'channels']],
    ];

    for (const [section, keys] of sections) {
      console.log(`  [${section}]`);
      for (const key of keys) {
        console.log(`    ${key}: ${JSON.stringify(args[key])}`);
      }
      console.log('');
    }

    console.log('All args received successfully. ✓');
  },
});
