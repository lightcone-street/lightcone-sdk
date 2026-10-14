// Compile the packed SDK from a directory outside the repository. This catches
// missing published files, undeclared dependencies, and global module collisions.
import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import path from 'node:path';
const root = path.resolve(import.meta.dirname, '..');
const temporary = await mkdtemp(path.join(tmpdir(), 'lightcone-rescript-consumer-'));
const run = (command, args, cwd) => execFileSync(command, args, { cwd, stdio: 'inherit' });
try {
  const packed = JSON.parse(execFileSync('npm', ['pack', '--ignore-scripts', '--json', '--pack-destination', temporary], { cwd: root, encoding: 'utf8' }));
  const consumer = path.join(temporary, 'app');
  await mkdir(path.join(consumer, 'src'), { recursive: true });
  await writeFile(path.join(consumer, 'package.json'), JSON.stringify({
    name: 'sdk-consumer-check', private: true, type: 'module',
    dependencies: { '@lightconexyz/lightcone-sdk-rescript': `file:${path.join(temporary, packed[0].filename)}` },
    devDependencies: { rescript: '12.3.0', typescript: '5.9.3' },
    trustedDependencies: ['@mununki/ppx-spice'],
  }));
  await writeFile(path.join(consumer, 'rescript.json'), JSON.stringify({
    name: 'sdk-consumer-check', sources: ['src'],
    dependencies: ['@lightconexyz/lightcone-sdk-rescript'],
    'package-specs': { module: 'esmodule', 'in-source': true }, suffix: '.res.mjs',
  }));
  await writeFile(path.join(consumer, 'src/Client.res'), 'let applicationName = "consumer"\n');
  await writeFile(path.join(consumer, 'src/Smoke.res'), [
    'let client = LightconeSdk.Client.make(~env=Staging, ())',
    'let applicationName = Client.applicationName',
    'let amount = LightconeSdk.ExactDecimal.scaledInteger("9007199254740993", 0)',
    '',
  ].join('\n'));
  await writeFile(path.join(consumer, 'smoke.ts'), [
    'import { makeForEnv } from "@lightconexyz/lightcone-sdk-rescript";',
    'const client = makeForEnv("staging");',
    'import {TransactionClient, SolClient, LiveWalletBalances, WsClient} from "@lightconexyz/lightcone-sdk-rescript";',
    'import {useNativeSigner, connect} from "@lightconexyz/lightcone-sdk-rescript/server";',
    'const checkServerConnection = () => WsClient.disconnect(connect(client, () => {}, undefined, undefined));',
    'TransactionClient.setResources(client, {computeUnitLimit: 200000, loadedAccountsDataSizeLimit: 1048576, priorityFeeLamports: 0n});',
    'void [client, SolClient, LiveWalletBalances, useNativeSigner];',
    '',
  ].join('\n'));
  await writeFile(path.join(consumer, 'tsconfig.json'), JSON.stringify({
    compilerOptions: { strict: true, target: 'ES2022', module: 'ESNext', moduleResolution: 'bundler', noEmit: true, skipLibCheck: false },
    include: ['smoke.ts'],
  }));
  run('bun', ['install'], consumer);
  run(path.join(consumer, 'node_modules/.bin/rescript'), [], consumer);
  run('node', ['src/Smoke.res.mjs'], consumer);
  run(path.join(consumer, 'node_modules/.bin/tsc'), ['-p', 'tsconfig.json'], consumer);
  run('node', ['--input-type=module', '-e', 'import {makeForEnv} from "@lightconexyz/lightcone-sdk-rescript"; makeForEnv("staging");'], consumer);
  console.log('Packed ReScript, TypeScript, and JavaScript consumers passed.');
} finally {
  if (process.env.KEEP_CONSUMER) console.log(`Consumer retained at ${temporary}`);
  else await rm(temporary, { recursive: true, force: true });
}
