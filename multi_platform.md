# 🌍 EyeMotion 跨平台开发与 CI/CD 坑点记录 (Multi-platform Pitfalls)

本文件记录了 EyeMotion 项目在不同平台开发、构建及 GitHub Actions CI/CD 流程中遇到的典型问题及其解决方案，供后续迭代参考。

---

## 🤖 Android (Tauri v2)

### 1. `tauri.settings.gradle` 缺失问题
*   **现象**：GitHub Actions 报错 `Could not read script .../tauri.settings.gradle' as it does not exist.`
*   **根源**：`tauri android init --ci` 在 CI 环境下经常无法正确生成该配置文件，导致 Gradle 无法定位 Tauri 的 Android 源码。
*   **最佳实践 (精确动态生成法)**：
    由于 `init` 的不稳定性，推荐在 `init` 之后使用 Python 脚本**精确查找** Cargo 注册表中的 `tauri` 和 `tauri-plugin-shell` 路径，并手动写回 `tauri.settings.gradle`。
    ```python
    # 核心正则匹配逻辑：确保只匹配 tauri-x.y.z 而不是 tauri-runtime-x.y.z
    matches = glob.glob(os.path.join(registry_path, f"{name_pattern}-[0-9]*"))
    exact = [m for m in matches if re.match(rf".*/{re.escape(name_pattern)}-\d+\.\d+", m)]
    ```
*   **解决方案总结**：
    1.  **彻底清理**：`rm -rf src-tauri/gen/android`。
    2.  **核心初始化**：`npx tauri android init --ci`。
    3.  **拉取源码缓存**：`cargo fetch --manifest-path src-tauri/Cargo.toml`（确保注册表中有源码供脚本查找）。
    4.  **精确补产配置文件**：运行上述正则匹配脚本。
    5.  **强制校验**：检查文件是否存在后再进入构建。

### 2. Gradle 构建任务不存在 (`Task not found`)
*   **现象**：报错 `Task 'assembleUniversalRelease' not found`。
*   **根源**：Tauri CLI v2 在运行 build 时，即使指定了单架构也会尝试寻找 `assembleUniversalRelease` 任务，而该任务只有在启用 `splits` 时才会存在。
*   **最佳实践 (CI 动态补丁法)**：
    由于 `npx tauri android init --ci` 在某些 CLI 版本中会覆盖整个 `build.gradle.kts`，最稳健的做法是在 init 之后利用 **Python 脚本** 动态注入 `splits` 配置。
    ```python
    import os
    path = "src-tauri/gen/android/app/build.gradle.kts"
    with open(path, "r") as f: content = f.read()
    if "splits {" not in content:
        content = content.replace("buildTypes {", "splits { abi { isEnable = true; reset(); include('arm64-v8a', 'armeabi-v7a', 'x86_64'); isUniversalApk = true } }\n    buildTypes {")
        with open(path, "w") as f: f.write(content)
    ```
    可在构建前添加验证步骤：
    ```bash
    grep -q "isUniversalApk" src-tauri/gen/android/app/build.gradle.kts || exit 1
    ```
*   **注意事项**：确保包含 `arm64-v8a`, `armeabi-v7a` 和 `x86_64` 以满足绝大多数需求，并设置 `isUniversalApk = true` 以生成 Tauri CLI 搜寻的特定 Task。

### 3. Tauri CLI 与 Rust Target 名称差异
*   **坑点**：`npx tauri android build --target` 接受的是**短名称**（如 `aarch64`），而 `rustup target add` 接受的是**完整名称**（如 `aarch64-linux-android`）。
*   **解决方案**：
    *   在 CI 安装 Rust 时使用：`aarch64-linux-android`
    *   在执行 `tauri android build` 时使用：`--target aarch64`
    *   **可选短名称列表**：`aarch64`, `armv7`, `i686`, `x86_64`

### 4. NDK 版本与环境变量
*   **现象**：Rust 编译 Android 库失败，报错找不到 NDK 或版本不匹配。
*   **最佳实践 (CI)**：
    1.  **版本锁定**：推荐使用 **NDK r27** (`27.2.12479018`)。
    2.  **环境变量注入**：必须显式将 `ANDROID_HOME` 和 `NDK_HOME` 写入 `$GITHUB_ENV`：
        ```yaml
        - name: Set Android environment variables
          run: |
            echo "ANDROID_HOME=$ANDROID_HOME" >> $GITHUB_ENV
            NDK_PATH=$(ls -d $ANDROID_HOME/ndk/* | sort -V | tail -1)
            echo "NDK_HOME=$NDK_PATH" >> $GITHUB_ENV
        ```
*   **根源**：GitHub Actions 的 ubuntu-latest 镜像预装的版本可能导致 Tauri 识别冲突，动态注入最新的 NDK 路径是最稳健的做法。

---

## 🪟 Windows (NSIS/MSI)

### 1. WebView2Loader.dll 缺失 (绿色版)
*   **现象**：打包出的 .zip 绿色版由于缺少 `WebView2Loader.dll` 无法在某些精简版系统运行。
*   **解决方案**：在 `build_all.ps1` 中显式从编译产物目录拷贝该 DLL 到便携包根目录。

---

## 🚀 GitHub Actions 策略建议

*   **版本控制 (SSOT)**：所有版本号变更必须同步修改 `package.json`, `src-tauri/tauri.conf.json`, 和根目录 `Cargo.toml`。
*   **Tag 驱动发布**：GitHub 工作流应由 `v*` 格式的 tag 触发，确保发布版本与代码库通过 Git 强对齐。
*   **强力清理 (Reset)**：如果构建多次失败导致 Tag 混乱，可以使用以下命令在本地重置并强推标签，保持远程状态纯净：
    ```powershell
    git tag -f <version>
    git push origin <version> --force
    ```
