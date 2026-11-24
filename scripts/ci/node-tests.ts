// Keep the instrumented and production Wasm tests in separate processes/builds.
import { spawnSync } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';

const root = fileURLToPath(new URL('../../', import.meta.url));
const output = resolve(root, 'target/coverage/node');
mkdirSync(output, { recursive: true });
let failed = false;

/**
 * Run `npm` through the Windows command interpreter: npm is a `.cmd` shim
 * there, which a direct spawn cannot execute.
 */
function npm(args: string[], directory = '.') {
  return process.platform === 'win32'
    ? run(process.env.ComSpec ?? 'cmd.exe', ['/d', '/s', '/c', 'npm', ...args], directory)
    : run('npm', args, directory);
}

function run(command: string, args: string[], directory = '.') {
  const result = spawnSync(command, args, {
    cwd: resolve(root, directory),
    stdio: 'inherit',
    env: { ...process.env, WRANGLER_SEND_METRICS: 'false' },
  });
  if (result.error) throw result.error;
  if (result.status !== 0) failed = true;
  return result.status === 0;
}

function test(name: string, files: string[], directory = '.') {
  run(
    'node',
    [
      '--test',
      '--experimental-test-coverage',
      '--test-reporter=spec',
      '--test-reporter-destination=stdout',
      '--test-reporter=lcov',
      `--test-reporter-destination=${resolve(output, `${name}.lcov`)}`,
      ...files,
    ],
    directory,
  );
}

// These tests were previously absent from the package test commands.
test('tooling', ['scripts/ci/*.test.ts']);
test('assets', ['scripts/assets/*.test.ts']);
test('cloudflare', ['scripts/cloudflare/*.test.ts']);
test('review-mount', ['scripts/local-review-files.test.ts'], 'apps/web');
test('wheel', ['bindings/python/scripts/*.test.ts']);
test('proxy', ['crates/usb-proxy/web/*.test.ts']);
// The standalone simulation's curated GLBs; the website never loads them.
test('simulation-assets', ['crates/simulation/*.test.ts']);
test('staging', ['bindings/javascript/scripts/*.test.ts']);
test('sharing', ['*.test.ts'], 'apps/usb-sharing');
for (const [name, testing] of [
  ['bindings-testing', true],
  ['bindings-production', false],
] as const) {
  if (!run('node', ['scripts/build.ts', ...(testing ? ['--testing'] : [])], 'bindings/javascript'))
    continue;
  if (!npm(['exec', '--', 'tsc'], 'bindings/javascript')) continue;
  npm(['exec', '--', 'tsc', '-p', 'tsconfig.types.json'], 'bindings/javascript');
  if (!npm(['exec', '--', 'tsc', '-p', 'tsconfig.scripts.json'], 'bindings/javascript'))
    continue;
  if (testing && !npm(['exec', '--', 'tsc', '-p', 'tsconfig.testing.json'], 'bindings/javascript'))
    continue;
  test(name, [testing ? 'tests/usb.test.ts' : 'tests/production.ts'], 'bindings/javascript');
}
process.exitCode = failed ? 1 : 0;
