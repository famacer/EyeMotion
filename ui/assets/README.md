# Asset Files Guide

## Directory Structure

Place your files in the following directories:

```
ui/
├── assets/
│   ├── fonts/
│   │   └── font.ttf           # Your custom English font file
│   └── icons/
│       ├── icon-16.png      # 16x16 icon (Windows taskbar)
│       ├── icon-32.png      # 32x32 icon (Windows desktop)
│       ├── icon-48.png      # 48x48 icon (Windows medium)
│       ├── icon-64.png      # 64x64 icon (Windows large)
│       ├── icon-128.png     # 128x128 icon (macOS)
│       ├── icon-256.png     # 256x256 icon (Windows extra large)
│       ├── icon-512.png     # 512x512 icon (High DPI)
│       └── icon-1024.png    # 1024x1024 icon (macOS icon)
```

## Icon Size Requirements

| Platform | Required Sizes |
|----------|---------------|
| Windows | 16x16, 32x32, 48x48, 64x64, 128x128, 256x256 |
| macOS | 128x128, 256x256, 512x512, 1024x1024 |
| Linux | 16x16, 32x32, 48x48, 64x64, 128x128 |
| Android | 48x48, 72x72, 96x96, 144x144, 192x192, 512x512 |
| iOS | 60x60, 76x76, 120x120, 152x152, 167x167, 180x180 |

## Icons Directory Location

**Full path**: `D:\Soft\eyemotion\ui\assets\icons\`

Please place your icon files in the directory above. After adding files, I will generate the necessary Tauri icon configurations for all platforms.
