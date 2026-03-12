import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { pickLocalIpv4 } from './local-ip.mjs';

const scriptDir = dirname(fileURLToPath(import.meta.url));
const debugXcconfigPath = resolve(scriptDir, '../ios/debug.xcconfig');
const host = process.env.CAP_DEV_HOST ?? pickLocalIpv4();
const port = process.env.CAP_DEV_PORT ?? '5173';

if (!host) {
  console.error(
    'Unable to determine a LAN IPv4 address. Set CAP_DEV_HOST manually, for example CAP_DEV_HOST=192.168.1.10.',
  );
  process.exit(1);
}

const existingConfig = readFileSync(debugXcconfigPath, 'utf8');
const rawServerUrl = process.env.CAP_SERVER_URL;
const serverHost = rawServerUrl ? new URL(rawServerUrl).hostname : host;
const serverPort = rawServerUrl ? new URL(rawServerUrl).port || (new URL(rawServerUrl).protocol === 'https:' ? '443' : '80') : port;

const nextHostLine = `CAP_SERVER_HOST = ${serverHost}`;
const nextPortLine = `CAP_SERVER_PORT = ${serverPort}`;
let nextConfig = existingConfig;

nextConfig = /^CAP_SERVER_HOST\s*=.*$/m.test(nextConfig)
  ? nextConfig.replace(/^CAP_SERVER_HOST\s*=.*$/m, nextHostLine)
  : `${nextConfig.trimEnd()}\n${nextHostLine}\n`;

nextConfig = /^CAP_SERVER_PORT\s*=.*$/m.test(nextConfig)
  ? nextConfig.replace(/^CAP_SERVER_PORT\s*=.*$/m, nextPortLine)
  : `${nextConfig.trimEnd()}\n${nextPortLine}\n`;

if (nextConfig !== existingConfig) {
  writeFileSync(debugXcconfigPath, nextConfig);
}

console.log(`Configured the fixed iOS Debug shell to use http://${serverHost}:${serverPort}`);
console.log('Run `task dev` in another terminal to serve JS updates to the app.');
