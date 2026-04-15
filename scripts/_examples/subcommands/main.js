'use strict';

const path = require('path');
const { createAgrrScript } = require(path.join(__dirname, '..', '..', '..', 'sdk', 'js', 'index.js'));

createAgrrScript({
  meta: {
    name: 'Subcommands Example (JS)',
    description: 'Demonstrates scripts with multiple subcommands, each with its own args',
    group: 'Examples',
    version: '1.0.0',
    subcommands: [
      {
        name: 'deploy',
        description: 'Deploy the application to an environment',
        args: [
          {
            name: 'environment',
            prompt: 'Target environment:',
            type: 'select',
            options: ['dev', 'staging', 'prod'],
            default: 'dev',
          },
          {
            name: 'version',
            prompt: 'Version to deploy (e.g. 1.2.3):',
            type: 'text',
            pattern: 'alphanumeric',
            max_length: 20,
          },
        ],
      },
      {
        name: 'rollback',
        description: 'Roll back to the previous deployment',
        args: [
          {
            name: 'environment',
            prompt: 'Target environment:',
            type: 'select',
            options: ['dev', 'staging', 'prod'],
          },
          {
            name: 'confirm',
            prompt: 'Confirm rollback:',
            type: 'select',
            options: ['yes', 'no'],
          },
        ],
      },
      {
        name: 'status',
        description: 'Show deployment status',
      },
    ],
  },

  subcommands: {
    async deploy({ args }) {
      const { environment = '', version = '' } = args;
      console.log(`Deploying version '${version}' to '${environment}'...`);
      console.log(`  environment : ${environment}`);
      console.log(`  version     : ${version}`);
      console.log('Deploy complete (example — no real action performed).');
    },

    async rollback({ args }) {
      const { environment = '', confirm = 'no' } = args;
      if (confirm !== 'yes') {
        console.log('Rollback cancelled.');
        return;
      }
      console.log(`Rolling back '${environment}'...`);
      console.log('Rollback complete (example — no real action performed).');
    },

    async status() {
      console.log('Deployment status:');
      console.log('  dev     : running v1.2.3');
      console.log('  staging : running v1.2.2');
      console.log('  prod    : running v1.2.1');
    },
  },
});
