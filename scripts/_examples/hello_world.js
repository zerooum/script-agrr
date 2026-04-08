'use strict';
/**
 * Exemplo mínimo de script JavaScript usando agrr-sdk.
 *
 * Coloque este arquivo em scripts/ e o agrr o descobrirá automaticamente.
 * Instale o SDK uma vez: npm link sdk/js
 */

const { createAgrrScript, AgrrAuthError } = require('agrr-sdk');

createAgrrScript({
  meta: {
    name: 'Hello World (JS)',
    description: 'Exibe uma saudação personalizada (Node.js)',
    group: 'exemplos',
    version: '1.0.0',

    // O campo abaixo demonstra autenticação.
    // Remova-o se o script não precisar de credenciais.
    requires_auth: ['GREETING_TOKEN'],

    args: [
      { name: 'name', prompt: 'Qual é o seu nome?' },
      {
        name: 'language',
        prompt: 'Idioma da saudação?',
        options: ['pt', 'en', 'es'],
      },
    ],
  },

  async run({ creds, args }) {
    const token = creds['GREETING_TOKEN'] ?? '';
    if (token !== 'valid-token') {
      // Simula rejeição de credencial — o CLI pedirá nova senha.
      throw new AgrrAuthError();
    }

    const name = args['name'] || 'Mundo';
    const language = args['language'] || 'pt';

    const greetings = { pt: 'Olá', en: 'Hello', es: 'Hola' };
    const greeting = greetings[language] ?? 'Olá';

    console.log(`${greeting}, ${name}!`);
  },
});
