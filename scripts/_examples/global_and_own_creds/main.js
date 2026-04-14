'use strict';

const path = require('path');
const { createAgrrScript, AgrrAuthError } = require(path.join(__dirname, '..', '..', '..', 'sdk', 'js', 'index.js'));

createAgrrScript({
  meta: {
    name: 'Global + Own Credentials Example (JS)',
    description: 'Example script using global credentials (CHAVE/SENHA) plus its own (ORG_TOKEN)',
    group: 'Examples',
    version: '1.0.0',
    global_auth: true,
    requires_auth: ['ORG_TOKEN'],
    args: [
      { name: 'resource', prompt: 'Resource name:', type: 'text' },
      {
        name: 'action',
        prompt: 'Action:',
        type: 'select',
        options: ['read', 'write', 'delete'],
        default: 'read',
      },
    ],
  },

  async run({ creds, args }) {
    const { CHAVE, SENHA, ORG_TOKEN } = creds;
    if (!CHAVE || !SENHA || !ORG_TOKEN) throw new AgrrAuthError();

    console.log('=== Global + Own Credentials Example ===\n');
    console.log('  Global credentials:');
    console.log(`    CHAVE:     ${JSON.stringify(CHAVE)}`);
    console.log(`    SENHA:     ${JSON.stringify(SENHA)}`);
    console.log('  Own credentials:');
    console.log(`    ORG_TOKEN: ${JSON.stringify(ORG_TOKEN)}`);
    console.log('  Args:');
    console.log(`    resource: ${JSON.stringify(args['resource'])}`);
    console.log(`    action:   ${JSON.stringify(args['action'])}`);
    console.log('\nAll credentials received successfully. ✓');
  },
});
