'use strict';

const path = require('path');
const { createAgrrScript, AgrrAuthError } = require(path.join(__dirname, '..', '..', '..', 'sdk', 'js', 'index.js'));

createAgrrScript({
  meta: {
    name: 'Global Credentials Example (JS)',
    description: 'Example script using global shared credentials (CHAVE and SENHA)',
    group: 'Examples',
    version: '1.0.0',
    global_auth: true,
    args: [
      {
        name: 'action',
        prompt: 'Action:',
        type: 'select',
        options: ['list', 'status', 'sync'],
        default: 'list',
      },
    ],
  },

  async run({ creds, args }) {
    const { CHAVE, SENHA } = creds;
    if (!CHAVE || !SENHA) throw new AgrrAuthError();

    console.log('=== Global Credentials Example ===\n');
    console.log(`  CHAVE:  ${JSON.stringify(CHAVE)}`);
    console.log(`  SENHA:  ${JSON.stringify(SENHA)}`);
    console.log(`  action: ${JSON.stringify(args['action'])}`);
    console.log('\nGlobal credentials received successfully. ✓');
  },
});
