'use strict';

const path = require('path');
const { createAgrrScript, AgrrAuthError } = require(path.join(__dirname, '..', '..', '..', 'sdk', 'js', 'index.js'));

createAgrrScript({
  meta: {
    name: 'Own Credentials Example (JS)',
    description: 'Example script that requires its own credentials (API_KEY and TOKEN)',
    group: 'Examples',
    version: '1.0.0',
    requires_auth: ['API_KEY', 'TOKEN'],
    args: [
      {
        name: 'action',
        prompt: 'Action:',
        type: 'select',
        options: ['list', 'create', 'delete'],
        default: 'list',
      },
    ],
  },

  async run({ creds, args }) {
    const { API_KEY, TOKEN } = creds;
    if (!API_KEY || !TOKEN) throw new AgrrAuthError();

    console.log('=== Own Credentials Example ===\n');
    console.log(`  API_KEY: ${JSON.stringify(API_KEY)}`);
    console.log(`  TOKEN:   ${JSON.stringify(TOKEN)}`);
    console.log(`  action:  ${JSON.stringify(args['action'])}`);
    console.log('\nCredentials received successfully. ✓');
  },
});
