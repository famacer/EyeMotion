const fs = require('fs');
const path = require('path');

const configPath = path.join(__dirname, 'src-tauri', 'tauri.conf.json');
const backupPath = path.join(__dirname, 'src-tauri', 'tauri.conf.json.bak');

console.log(`[Patch] Target config: ${configPath}`);

// 1. Create Backup
try {
    fs.copyFileSync(configPath, backupPath);
    console.log('[Patch] Backup created at:', backupPath);
} catch (e) {
    console.error('[Patch] Failed to create backup:', e);
    process.exit(1);
}

// 2. Read Config
let configRaw;
try {
    configRaw = fs.readFileSync(configPath, 'utf8');
} catch (e) {
    console.error('[Patch] Failed to read config:', e);
    process.exit(1);
}

let config;
try {
    config = JSON.parse(configRaw);
} catch (e) {
    console.error('[Patch] Failed to parse JSON:', e);
    process.exit(1);
}

// 3. Modify Config
if (config.build) {
    console.log('[Patch] Found build configuration.');
    if (config.build.devUrl) {
        console.log(`[Patch] Removing devUrl: ${config.build.devUrl}`);
        delete config.build.devUrl;
    } else {
        console.log('[Patch] No devUrl found to remove.');
    }
} else {
    console.warn('[Patch] No build configuration found!');
}

// 4. Save Config
try {
    fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
    console.log('[Patch] Successfully updated tauri.conf.json (devUrl removed)');
} catch (e) {
    console.error('[Patch] Failed to write config:', e);
    process.exit(1);
}
