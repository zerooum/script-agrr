'use strict';

const path = require('path');
// Resolve o SDK a partir da raiz do projeto para funcionar sem npm link.
const { createAgrrScript } = require(path.join(__dirname, '..', 'sdk', 'js', 'index.js'));

createAgrrScript({
  meta: {
    name: 'Hello World (JS)',
    description: 'Exibe uma saudação — script de teste Node.js',
    group: 'exemplos',
    version: '1.0.0',
    runtime: { language: 'node', min_version: '18' },
    args: [
      { name: 'nome', prompt: 'Qual é o seu nome?', type: 'text' },
      {
        name: 'idioma',
        prompt: 'Idioma da saudação?',
        type: 'select',
        options: ['pt', 'en', 'es'],
        default: 'pt',
      },
    ],
  },

  async run({ args }) {
    const nome = args['nome'] || 'Mundo';
    const idioma = args['idioma'] || 'pt';

    const saudacoes = { pt: 'Olá', en: 'Hello', es: 'Hola' };
    const saudacao = saudacoes[idioma] ?? 'Olá';

    console.log(`${saudacao}, ${nome}!`);
    console.log('Script JS executado com sucesso. ✓');
  },
});
