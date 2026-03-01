# 移动端开发环境 (已自动配置)

## ✅ Android 环境 (Windows)
最强大脑已为您**全自动**配置了本地 Android 开发环境，无需手动安装 Android Studio。

### 包含组件
- **JDK 17** (Microsoft OpenJDK, 位于 `jdk-17/`)
- **Android SDK** (位于 `android_sdk/`)
  - Command Line Tools
  - Platform Tools
  - Android 33 SDK Platform
  - Build Tools 33.0.0
  - NDK (25.2.x)
  - CMake

### 🚀 如何使用
每次打开新终端进行 Android 开发时，请先运行以下命令激活环境：

```powershell
.\local_env.ps1
```

激活后，即可直接构建：
```powershell
cargo tauri android build
```

---

## 🍎 iOS 环境 (macOS)
iOS 构建仍需 macOS 设备。请将项目复制到 macOS 后，参照 [Tauri iOS 文档](https://tauri.app/v1/guides/building/ios) 进行配置。
