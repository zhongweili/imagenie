#!/usr/bin/env node

import fs from "fs/promises";
import { existsSync, mkdirSync } from "fs";
import path from "path";
const getFiles = async (dir) => {
  try {
    const files = await fs.readdir(dir);
    return files;
  } catch (err) {
    console.error(`Error reading directory ${dir}:`, err);
    return [];
  }
};
const copyFile = async (from, to) => {
  if (!existsSync(from)) {
    console.error(`Source file does not exist: ${from}`);
    return false;
  }
  const dirname = path.dirname(to);
  if (!existsSync(dirname)) {
    mkdirSync(dirname, { recursive: true });
  }
  await fs.copyFile(from, to).catch(() => null);
};
async function main() {
  try {
    const args = process.argv.slice(2);
    const [target, appName] = args;
    const bundleDir = path.resolve(`src-tauri/target/${target}/release/bundle`);

    if (!existsSync(bundleDir)) {
      console.error(`Build directory not found: ${bundleDir}`);
      process.exit(1);
    }

    let outputs = {};
    switch (process.platform) {
      case "darwin":
        outputs = {
          dmg: [".dmg"],
        };
        break;
      case "win32":
        const nsisDir = path.join(bundleDir, 'nsis');
        if (!existsSync(nsisDir)) {
          console.error('NSIS bundle directory not found. Make sure NSIS is installed and the build completed successfully.');
          console.error(`Expected directory: ${nsisDir}`);
          process.exit(1);
        }
        outputs = {
          nsis: [".exe"],
        };
        break;
    }
    for (const dir in outputs) {
      const files = await getFiles(path.join(bundleDir, dir));
      for (const filename of files) {
        const suffix = outputs[dir].find((e) => filename.endsWith(e));
        if (suffix) {
          await copyFile(
            path.join(bundleDir, dir, filename),
            path.join("dist", appName + suffix)
          );
          console.log(`✅ ${appName + suffix}`);
        }
      }
    }
  } catch (error) {
    console.error('Error in dist script:', error);
    process.exit(1);
  }
}
main();
