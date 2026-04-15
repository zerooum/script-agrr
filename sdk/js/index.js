'use strict';

const { execSync } = require('child_process');

/**
 * Thrown inside `run()` to signal authentication failure.
 * The agrr CLI will delete stored credentials and re-prompt the user.
 *
 * @example
 * const { createAgrrScript, AgrrAuthError } = require('agrr-sdk');
 * createAgrrScript({
 *   meta: { ... },
 *   async run({ creds }) {
 *     if (!await login(creds.SERVICE_USER, creds.SERVICE_PASS)) {
 *       throw new AgrrAuthError();
 *     }
 *   },
 * });
 */
class AgrrAuthError extends Error {
  constructor() {
    super('authentication failed');
    this.name = 'AgrrAuthError';
  }
}

/**
 * Collect credentials from environment variables injected by the CLI.
 * Each key in `requiresAuth` maps to `AGRR_CRED_<UPPERCASE_KEY>`.
 *
 * @param {string[]} requiresAuth
 * @returns {Record<string, string>}
 */
function collectCreds(requiresAuth) {
  const creds = {};
  for (const key of requiresAuth) {
    creds[key] = process.env[`AGRR_CRED_${key.toUpperCase()}`] ?? '';
  }
  return creds;
}

/**
 * Collect args from environment variables injected by the CLI.
 * Each arg `name` maps to `AGRR_ARG_<UPPERCASE_NAME>`.
 *
 * @param {{ name: string }[]} argSpecs
 * @returns {Record<string, string>}
 */
function collectArgs(argSpecs) {
  const args = {};
  for (const spec of argSpecs) {
    args[spec.name] = process.env[`AGRR_ARG_${spec.name.toUpperCase()}`] ?? '';
  }
  return args;
}

/**
 * Define an agrr-compatible script.
 *
 * The returned object handles `--agrr-meta` and `--agrr-run` dispatch
 * automatically when this module is the entry point.
 *
 * @param {object} options
 * @param {object} options.meta - Script metadata (required fields: name, description, group, version).
 * @param {string} options.meta.name
 * @param {string} options.meta.description
 * @param {string} options.meta.group
 * @param {string} options.meta.version
 * @param {{ language: 'python'|'node', min_version: string }} [options.meta.runtime]
 * @param {string[]} [options.meta.requires_auth]
 * @param {boolean} [options.meta.global_auth] - If true, CHAVE and SENHA global credentials are injected.
 * @param {{
 *   name: string,
 *   prompt: string,
 *   type: 'text'|'select'|'multiselect',
 *   options?: string[],
 *   max_length?: number,
 *   pattern?: 'numeric'|'alpha'|'alphanumeric',
 *   required?: boolean,
 *   default?: string
 * }[]} [options.meta.args] - Arg specs. `type` is required. `select`/`multiselect` require
 *   `options` with ≥ 2 entries. For `multiselect`, the selected values arrive as a
 *   comma-separated string in `AGRR_ARG_<NAME>`. `max_length`, `pattern` apply to `text` only.
 * @param {(ctx: { creds: Record<string,string>, args: Record<string,string> }) => Promise<void>|void} options.run
 *
 * @example
 * const { createAgrrScript, AgrrAuthError } = require('agrr-sdk');
 *
 * createAgrrScript({
 *   meta: {
 *     name: 'Deploy Produção',
 *     description: 'Faz deploy via AWS',
 *     group: 'infra',
 *     version: '1.0.0',
 *     requires_auth: ['AWS_USER', 'AWS_PASS'],
 *     args: [
 *       { name: 'env', prompt: 'Ambiente?', type: 'select', options: ['prod', 'staging'] },
 *       { name: 'tags', prompt: 'Tags?', type: 'multiselect', options: ['alpha', 'beta', 'rc'] },
 *     ],
 *   },
 *   async run({ creds, args }) {
 *     const tags = args.tags ? args.tags.split(',') : [];
 *     if (!await deploy(creds.AWS_USER, creds.AWS_PASS, args.env, tags)) {
 *       throw new AgrrAuthError();
 *     }
 *   },
 * });
 */
function createAgrrScript({ meta, run, subcommands }) {
  const argv = process.argv.slice(2);

  // Validate: either run or subcommands must be provided (but not both)
  const hasRun = typeof run === 'function';
  const hasSubcmds = subcommands && typeof subcommands === 'object' && Object.keys(subcommands).length > 0;

  if (!hasRun && !hasSubcmds) {
    process.stderr.write("agrr-sdk: 'run' function or 'subcommands' object not provided\n");
    process.exit(1);
  }

  if (argv.includes('--agrr-meta')) {
    // Validate subcommand handlers
    if (meta.subcommands && meta.subcommands.length > 0) {
      for (const spec of meta.subcommands) {
        if (typeof (subcommands && subcommands[spec.name]) !== 'function') {
          process.stderr.write(`agrr-sdk: subcommand '${spec.name}' has no matching handler\n`);
          process.exit(1);
        }
      }
    }
    const output = {
      name: meta.name,
      description: meta.description,
      group: meta.group,
      version: meta.version,
    };
    if (meta.runtime) output.runtime = meta.runtime;
    if (meta.requires_auth && meta.requires_auth.length > 0) {
      output.requires_auth = meta.requires_auth;
    }
    if (meta.args && meta.args.length > 0) {
      output.args = meta.args;
    }
    if (meta.subcommands && meta.subcommands.length > 0) {
      output.subcommands = meta.subcommands;
    }
    if (meta.global_auth) {
      output.global_auth = true;
    }
    process.stdout.write(JSON.stringify(output) + '\n');
    process.exit(0);
  }

  if (argv.includes('--agrr-run')) {
    const creds = collectCreds(meta.requires_auth ?? []);
    if (meta.global_auth) {
      for (const key of ['CHAVE', 'SENHA']) {
        creds[key] = process.env[`AGRR_CRED_${key}`] ?? '';
      }
    }

    const subcommandName = process.env.AGRR_SUBCOMMAND;
    let handler;
    let argSpecs;

    if (subcommandName !== undefined) {
      handler = subcommands && subcommands[subcommandName];
      if (typeof handler !== 'function') {
        console.error(`agrr-sdk: unknown subcommand '${subcommandName}'`);
        process.exit(1);
      }
      const subcmdSpec = (meta.subcommands ?? []).find(s => s.name === subcommandName);
      argSpecs = subcmdSpec ? (subcmdSpec.args ?? []) : [];
    } else {
      handler = run;
      argSpecs = meta.args ?? [];
    }

    const args = collectArgs(argSpecs);

    Promise.resolve()
      .then(() => handler({ creds, args }))
      .then(() => {
        process.exit(0);
      })
      .catch((err) => {
        if (err instanceof AgrrAuthError) {
          process.exit(99);
        }
        console.error(`Error: ${err && err.message ? err.message : err}`);
        process.exit(1);
      });
    return;
  }

  console.error('agrr-sdk: use --agrr-meta or --agrr-run');
  process.exit(1);
}

module.exports = { createAgrrScript, AgrrAuthError };
