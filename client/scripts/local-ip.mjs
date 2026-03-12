import { spawnSync } from 'node:child_process';
import { networkInterfaces } from 'node:os';

function isPrivateIpv4(address) {
  return (
    address.startsWith('10.') ||
    address.startsWith('192.168.') ||
    /^172\.(1[6-9]|2\d|3[01])\./.test(address)
  );
}

function getExternalIpv4AddressesByInterface() {
  const interfaces = networkInterfaces();
  const addressesByInterface = new Map();

  for (const [name, addresses] of Object.entries(interfaces)) {
    if (!addresses) {
      continue;
    }

    for (const address of addresses) {
      const family = typeof address.family === 'string' ? address.family : address.family === 4 ? 'IPv4' : 'IPv6';

      if (family !== 'IPv4' || address.internal || address.address.startsWith('169.254.')) {
        continue;
      }

      const existingAddresses = addressesByInterface.get(name) ?? [];
      existingAddresses.push(address.address);
      addressesByInterface.set(name, existingAddresses);
    }
  }

  return addressesByInterface;
}

function parseDeviceForHardwarePort(output, hardwarePortName) {
  const blocks = output.split(/\n\s*\n/);

  for (const block of blocks) {
    const hardwarePortMatch = block.match(/^Hardware Port: (.+)$/m);
    const deviceMatch = block.match(/^Device: (.+)$/m);

    if (!hardwarePortMatch || !deviceMatch) {
      continue;
    }

    if (hardwarePortMatch[1] === hardwarePortName) {
      return deviceMatch[1];
    }
  }

  return undefined;
}

function pickWifiIpv4(addressesByInterface) {
  const result = spawnSync('networksetup', ['-listallhardwareports'], {
    encoding: 'utf8',
  });

  if (result.status !== 0 || !result.stdout) {
    return undefined;
  }

  const wifiDevice = parseDeviceForHardwarePort(result.stdout, 'Wi-Fi');
  if (!wifiDevice) {
    return undefined;
  }

  return addressesByInterface.get(wifiDevice)?.find(isPrivateIpv4);
}

function pickDefaultRouteIpv4(addressesByInterface) {
  const result = spawnSync('route', ['-n', 'get', 'default'], {
    encoding: 'utf8',
  });

  if (result.status !== 0 || !result.stdout) {
    return undefined;
  }

  const interfaceMatch = result.stdout.match(/interface: (.+)/);
  if (!interfaceMatch) {
    return undefined;
  }

  const routeInterface = interfaceMatch[1].trim();
  return addressesByInterface.get(routeInterface)?.find(isPrivateIpv4);
}

export function pickLocalIpv4() {
  const addressesByInterface = getExternalIpv4AddressesByInterface();
  const candidates = Array.from(addressesByInterface.values()).flat();

  const wifiAddress = pickWifiIpv4(addressesByInterface);
  if (wifiAddress) {
    return wifiAddress;
  }

  const defaultRouteAddress = pickDefaultRouteIpv4(addressesByInterface);
  if (defaultRouteAddress) {
    return defaultRouteAddress;
  }

  return candidates.find(isPrivateIpv4) ?? candidates[0];
}
