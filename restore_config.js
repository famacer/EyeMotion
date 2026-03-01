const fs = require('fs');
const path = require('path');

const configPath = path.join(__dirname, 'src-tauri', 'tauri.conf.json');
const backupPath = path.join(__dirname, 'src-tauri', 'tauri.conf.json.bak');

console.log(`[Restore] Target config: ${configPath}`);

if (fs.existsSync(backupPath)) {
    try {
        fs.copyFileSync(backupPath, configPath);
        console.log('[Restore] Restored tauri.conf.json from backup');
        
        // Optional: Delete backup after restore
        fs.unlinkSync(backupPath);
        console.log('[Restore] Backup file deleted');
    } catch (e) {
        console.error('[Restore] Failed to restore config:', e);
        process.exit(1);
    }
} else {
    console.warn('[Restore] No backup found to restore!');
    // Don't fail here, as we might have just failed before patching
}
