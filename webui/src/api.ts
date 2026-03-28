import * as ksu from 'kernelsu';

export interface AppInfo {
  packageName: string;
  appName: string;
  iconUrl: string;
  isEnabled: boolean;
  processes: string[]; // From config file `pkg|process`
}

const CONFIG_PATH = "/data/adb/hmspush/app.conf";

export type ConfigMap = Map<string, string[]>;

/**
 * Loads and parses the configuration file.
 */
export async function loadConfig(): Promise<ConfigMap> {
  const configMap: ConfigMap = new Map();
  try {
    const result = await ksu.exec(`cat ${CONFIG_PATH}`);
    if (result.errno === 0 && result.stdout) {
      const lines = result.stdout.split('\n');
      for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed || trimmed.startsWith('#')) continue;

        const [packageName, processName] = trimmed.split('|');
        if (packageName) {
          const procs = configMap.get(packageName) || [];
          if (processName) procs.push(processName);
          configMap.set(packageName, procs);
        }
      }
    }
  } catch (e) {
    console.error("Failed to load config", e);
  }
  return configMap;
}

/**
 * Writes the configuration map to the file.
 */
export async function writeConfig(configMap: ConfigMap): Promise<void> {
  let content = '';
  for (const [pkg, processes] of configMap.entries()) {
    if (processes.length === 0) {
      content += `${pkg}\n`;
    } else {
      for (const proc of processes) {
        content += `${pkg}|${proc}\n`;
      }
    }
  }

  try {
    const cmd = `mkdir -p $(dirname ${CONFIG_PATH}) && echo '${content.trim()}' > ${CONFIG_PATH}`;
    const result = await ksu.exec(cmd);
    if (result.errno !== 0) {
      console.error("Failed to write config", result.stderr);
      ksu.toast("Failed to update config");
    }
  } catch (e) {
    console.error("Exec error writing config", e);
    ksu.toast("Error writing config");
  }
}

/**
 * Queries packages by intent actions and merges with config.
 */
export async function getAppsData(): Promise<AppInfo[]> {
  const configMap = await loadConfig();
  const foundPackages = new Set<string>();

  // Add packages from config
  for (const pkg of configMap.keys()) {
    foundPackages.add(pkg);
  }

  try {
    // Query receivers
    const receiversRes = await ksu.exec('cmd package query-receivers --components -a com.huawei.android.push.intent.REGISTRATION');
    if (receiversRes.errno === 0) {
      const pkgs = extractPackagesFromCmd(receiversRes.stdout);
      pkgs.forEach(p => foundPackages.add(p));
    }

    // Query services
    const servicesRes = await ksu.exec('cmd package query-services --components -a com.huawei.push.msg.NOTIFY_MSG');
    if (servicesRes.errno === 0) {
      const pkgs = extractPackagesFromCmd(servicesRes.stdout);
      pkgs.forEach(p => foundPackages.add(p));
    }
  } catch (e) {
    console.error("Error querying packages via cmd", e);
  }

  const packageList = Array.from(foundPackages);
  if (packageList.length === 0) return [];

  // Get app labels
  const packagesInfo = ksu.getPackagesInfo(packageList);
  const infoMap = new Map(packagesInfo.map(info => [info.packageName, info.appLabel]));

  // Build final AppInfo array
  return packageList.map(pkg => ({
    packageName: pkg,
    appName: infoMap.get(pkg) || pkg,
    iconUrl: `ksu://icon/${pkg}`,
    isEnabled: configMap.has(pkg),
    processes: configMap.get(pkg) || [],
  })).sort((a, b) => {
    // Sort enabled first, then alphabetically by app name
    if (a.isEnabled && !b.isEnabled) return -1;
    if (!a.isEnabled && b.isEnabled) return 1;
    return a.appName.localeCompare(b.appName);
  });
}

function extractPackagesFromCmd(output: string): string[] {
  const pkgs: string[] = [];
  // Output format example: 
  //   com.example.app/com.example.receiver
  // or something similar containing the package name followed by a slash.
  const lines = output.split('\n');
  for (const line of lines) {
    const match = line.match(/([a-zA-Z0-9_.]+)\//);
    if (match && match[1]) {
      pkgs.push(match[1]);
    }
  }
  return pkgs;
}
