'use strict';

/**
 * Tests for the agrr JavaScript SDK.
 *
 * Uses Node's built-in test runner (node --test, available since v18).
 * Run with: node --test tests/
 */

const { test, describe } = require('node:test');
const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');
const path = require('node:path');
const fs = require('node:fs');
const os = require('node:os');

// ─── Helpers ──────────────────────────────────────────────────────────────────

const SDK_PATH = path.resolve(__dirname, '..');

/**
 * Write a minimal script file that uses createAgrrScript and run it with
 * the given argv and environment variables.
 */
function runScript(scriptContent, argv, env = {}) {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'agrr-sdk-test-'));
  const scriptPath = path.join(tmpDir, 'script.js');
  fs.writeFileSync(scriptPath, scriptContent);

  const result = spawnSync('node', [scriptPath, ...argv], {
    env: { ...process.env, ...env },
    encoding: 'utf8',
    timeout: 5000,
  });

  fs.rmSync(tmpDir, { recursive: true, force: true });
  return result;
}

function makeScript(meta, runBody = '') {
  return `
'use strict';
const { createAgrrScript, AgrrAuthError } = require(${JSON.stringify(SDK_PATH)});
createAgrrScript({
  meta: ${JSON.stringify(meta)},
  async run({ creds, args }) {
    ${runBody}
  },
});
`;
}

// ─── Meta dispatch ────────────────────────────────────────────────────────────

describe('--agrr-meta dispatch', () => {
  const META = {
    name: 'Deploy',
    description: 'Deploys the app',
    group: 'infra',
    version: '2.0.0',
  };

  test('outputs valid JSON manifest', () => {
    const result = runScript(makeScript(META), ['--agrr-meta']);
    assert.equal(result.status, 0, `Expected exit 0, got ${result.status}: ${result.stderr}`);
    const data = JSON.parse(result.stdout.trim());
    assert.equal(data.name, 'Deploy');
    assert.equal(data.version, '2.0.0');
    assert.equal(data.group, 'infra');
  });

  test('exits with code 0', () => {
    const result = runScript(makeScript(META), ['--agrr-meta']);
    assert.equal(result.status, 0);
  });

  test('includes requires_auth when non-empty', () => {
    const meta = { ...META, requires_auth: ['API_KEY', 'API_SECRET'] };
    const result = runScript(makeScript(meta), ['--agrr-meta']);
    const data = JSON.parse(result.stdout.trim());
    assert.deepEqual(data.requires_auth, ['API_KEY', 'API_SECRET']);
  });

  test('omits requires_auth when empty', () => {
    const meta = { ...META, requires_auth: [] };
    const result = runScript(makeScript(meta), ['--agrr-meta']);
    const data = JSON.parse(result.stdout.trim());
    assert.equal('requires_auth' in data, false, 'requires_auth should be omitted when empty');
  });

  test('includes args when non-empty', () => {
    const meta = { ...META, args: [{ name: 'env', prompt: 'Env?', type: 'select', options: ['prod', 'staging'] }] };
    const result = runScript(makeScript(meta), ['--agrr-meta']);
    const data = JSON.parse(result.stdout.trim());
    assert.equal(data.args[0].name, 'env');
  });

  test('includes runtime when set', () => {
    const meta = { ...META, runtime: { language: 'node', min_version: '18' } };
    const result = runScript(makeScript(meta), ['--agrr-meta']);
    const data = JSON.parse(result.stdout.trim());
    assert.equal(data.runtime.language, 'node');
    assert.equal(data.runtime.min_version, '18');
  });
});

// ─── Run dispatch ─────────────────────────────────────────────────────────────

describe('--agrr-run dispatch', () => {
  test('exits with code 0 on success', () => {
    const result = runScript(makeScript({ name: 'S', description: 'D', group: 'g', version: '1.0.0' }), ['--agrr-run']);
    assert.equal(result.status, 0);
  });

  test('exits with code 99 on AgrrAuthError', () => {
    const meta = { name: 'S', description: 'D', group: 'g', version: '1.0.0' };
    const body = `throw new AgrrAuthError();`;
    const result = runScript(makeScript(meta, body), ['--agrr-run']);
    assert.equal(result.status, 99);
  });

  test('exits with code 1 on unexpected error', () => {
    const meta = { name: 'S', description: 'D', group: 'g', version: '1.0.0' };
    const body = `throw new Error('something broke');`;
    const result = runScript(makeScript(meta, body), ['--agrr-run']);
    assert.equal(result.status, 1);
  });

  test('injects credentials from AGRR_CRED_* env vars', () => {
    const meta = {
      name: 'S', description: 'D', group: 'g', version: '1.0.0',
      requires_auth: ['DB_PASS'],
    };
    const body = `process.stdout.write('got:' + creds.DB_PASS);`;
    const result = runScript(makeScript(meta, body), ['--agrr-run'], { AGRR_CRED_DB_PASS: 'secret' });
    assert.match(result.stdout, /got:secret/);
  });

  test('injects args from AGRR_ARG_* env vars', () => {
    const meta = {
      name: 'S', description: 'D', group: 'g', version: '1.0.0',
      args: [{ name: 'target', prompt: 'Target?', type: 'text' }],
    };
    const body = `process.stdout.write('target:' + args.target);`;
    const result = runScript(makeScript(meta, body), ['--agrr-run'], { AGRR_ARG_TARGET: 'staging' });
    assert.match(result.stdout, /target:staging/);
  });
});

// ─── No-flag fallback ─────────────────────────────────────────────────────────

describe('no-flag fallback', () => {
  test('exits with code 1 when no flags given', () => {
    const result = runScript(makeScript({ name: 'S', description: 'D', group: 'g', version: '1.0.0' }), []);
    assert.equal(result.status, 1);
  });
});

// ─── AgrrAuthError class ─────────────────────────────────────────────────────

describe('AgrrAuthError', () => {
  test('is an instance of Error', () => {
    const { AgrrAuthError } = require(SDK_PATH);
    const err = new AgrrAuthError();
    assert.ok(err instanceof Error);
    assert.equal(err.name, 'AgrrAuthError');
  });
});

// ─── Arg constraint field tests ───────────────────────────────────────────────

describe('arg constraint fields in meta', () => {
  const BASE = { name: 'S', description: 'D', group: 'g', version: '1.0.0' };

  test('arg type field is serialized to meta JSON', () => {
    const meta = { ...BASE, args: [{ name: 'x', prompt: 'X?', type: 'text' }] };
    const result = runScript(makeScript(meta), ['--agrr-meta']);
    const data = JSON.parse(result.stdout.trim());
    assert.equal(data.args[0].type, 'text');
  });

  test('select arg with options is serialized', () => {
    const meta = { ...BASE, args: [{ name: 'env', prompt: 'Env?', type: 'select', options: ['prod', 'staging'] }] };
    const result = runScript(makeScript(meta), ['--agrr-meta']);
    const data = JSON.parse(result.stdout.trim());
    assert.equal(data.args[0].type, 'select');
    assert.deepEqual(data.args[0].options, ['prod', 'staging']);
  });

  test('multiselect arg is serialized', () => {
    const meta = { ...BASE, args: [{ name: 'tags', prompt: 'Tags?', type: 'multiselect', options: ['a', 'b', 'c'] }] };
    const result = runScript(makeScript(meta), ['--agrr-meta']);
    const data = JSON.parse(result.stdout.trim());
    assert.equal(data.args[0].type, 'multiselect');
    assert.equal(data.args[0].options.length, 3);
  });

  test('text arg constraints are serialized', () => {
    const meta = { ...BASE, args: [{ name: 'code', prompt: 'Code?', type: 'text', max_length: 6, pattern: 'numeric', required: false, default: '000' }] };
    const result = runScript(makeScript(meta), ['--agrr-meta']);
    const data = JSON.parse(result.stdout.trim());
    const arg = data.args[0];
    assert.equal(arg.max_length, 6);
    assert.equal(arg.pattern, 'numeric');
    assert.equal(arg.required, false);
    assert.equal(arg.default, '000');
  });

  test('multiselect values arrive comma-separated via env var', () => {
    const meta = { ...BASE, args: [{ name: 'tags', prompt: 'Tags?', type: 'multiselect', options: ['alpha', 'beta', 'rc'] }] };
    const body = `process.stdout.write('tags:' + args.tags);`;
    const result = runScript(makeScript(meta, body), ['--agrr-run'], { AGRR_ARG_TAGS: 'alpha,rc' });
    assert.match(result.stdout, /tags:alpha,rc/);
  });
});

// ─── Run contract validation ──────────────────────────────────────────────────

describe('run contract validation', () => {
  const META = { name: 'S', description: 'D', group: 'g', version: '1.0.0' };

  test('--agrr-meta exits 1 when run is not provided', () => {
    const script = `
'use strict';
const { createAgrrScript } = require(${JSON.stringify(SDK_PATH)});
createAgrrScript({ meta: ${JSON.stringify(META)} });
`;
    const result = runScript(script, ['--agrr-meta']);
    assert.equal(result.status, 1, `Expected exit 1, got ${result.status}: ${result.stdout}`);
    assert.match(result.stderr, /agrr-sdk: 'run' function not provided/);
  });

  test('--agrr-meta exits 1 when run is null', () => {
    const script = `
'use strict';
const { createAgrrScript } = require(${JSON.stringify(SDK_PATH)});
createAgrrScript({ meta: ${JSON.stringify(META)}, run: null });
`;
    const result = runScript(script, ['--agrr-meta']);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /agrr-sdk: 'run' function not provided/);
  });

  test('--agrr-meta exits 0 when run is a valid function', () => {
    const result = runScript(makeScript(META), ['--agrr-meta']);
    assert.equal(result.status, 0, `Expected exit 0, got ${result.status}: ${result.stderr}`);
    const data = JSON.parse(result.stdout.trim());
    assert.equal(data.name, META.name);
  });
});
