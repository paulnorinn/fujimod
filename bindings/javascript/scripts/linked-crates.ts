import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { basename, dirname } from 'node:path';

type CargoMessage = {
  reason: string;
  message?: { rendered?: string };
  manifest_path?: string;
  filenames?: string[];
  target?: { name?: string };
};

/** The artifact stems tie linker entries to the exact Cargo package, including parallel versions. */
export function cargoArtifacts(
  args: string[],
  cwd: string,
  env: NodeJS.ProcessEnv,
  rustcArgs: string[] = [],
) {
  const result = spawnSync(
    'cargo',
    [
      ...args,
      '--message-format=json-render-diagnostics',
      ...(rustcArgs.length ? ['--', ...rustcArgs] : []),
    ],
    {
      cwd,
      env,
      encoding: 'utf8',
      stdio: ['inherit', 'pipe', 'inherit'],
      maxBuffer: 32 * 1024 * 1024,
    },
  );
  if (result.error) throw result.error;
  const artifacts = new Map<string, string>();
  for (const line of result.stdout.split('\n')) {
    if (!line.startsWith('{')) continue;
    const message: CargoMessage = JSON.parse(line);
    if (message.reason === 'compiler-message' && message.message?.rendered)
      process.stderr.write(message.message.rendered);
    if (message.reason !== 'compiler-artifact' || !message.manifest_path) continue;
    // A binary crate (the wasm module itself) emits no rlib, so record its
    // target name as well; the linker map names its objects after it.
    if (message.target?.name) artifacts.set(message.target.name, dirname(message.manifest_path));
    for (const artifact of message.filenames ?? []) {
      // The artifact is `lib<crate>-<hash>.rlib` normally, but a crate whose
      // dependencies change every build (the wasm bindings embed the generated
      // schema) is emitted as plain `lib<crate>.rlib`.
      const match = /^lib([\w]+?)(?:-([a-f0-9]+))?\.rlib$/.exec(basename(artifact));
      if (!match) continue;
      const directory = dirname(message.manifest_path);
      if (match[2]) artifacts.set(`${match[1]}-${match[2]}`, directory);
      // Windows thin-LTO objects are named after the crate alone, so record the
      // crate name too and let the linker map resolve them by it.
      artifacts.set(match[1], directory);
    }
  }
  if (result.status !== 0) throw Error(`Cargo exited ${result.status ?? result.signal}`);
  return artifacts;
}

export function linkedCrates(map: string, artifacts: ReadonlyMap<string, string>) {
  const directories = new Set<string>();
  // Nonzero linker-map entries cover object members of the emulator's static
  // archive and rlibs linked directly by rustc into standalone Wasm modules.
  const rows = /^ *[-\da-f]+ +[\da-f]+ +([\da-f]+) +([^\r\n]+)/gm;
  for (const [, size, input] of map.matchAll(rows)) {
    if (!Number.parseInt(size, 16)) continue;
    const stem =
      /(?:^|[/\\])lib([\w]+-[a-f0-9]+)\.rlib\(/.exec(input)?.[1] ??
      /\.a\(([\w]+-[a-f0-9]+)\.[^()]*\.rcgu\.o\)/.exec(input)?.[1] ??
      /[/\\]([\w]+-[a-f0-9]+)\.[^()]*\.rcgu\.o:/.exec(input)?.[1] ??
      // Windows: `<crate>.<codegen hash>.<object hash>.rcgu.o`, no rlib hash.
      /[/\\]([\w]+)\.[0-9a-z]+\.[0-9a-z]+\.rcgu\.o:/.exec(input)?.[1];
    const directory = stem && artifacts.get(stem);
    if (directory) directories.add(directory);
  }
  if (!directories.size)
    throw Error('No Cargo artifacts matched emitted bytes in the Wasm linker map');
  return [...directories].sort();
}

export function readLinkedCrates(map: string, artifacts: ReadonlyMap<string, string>) {
  return linkedCrates(readFileSync(map, 'utf8'), artifacts);
}
