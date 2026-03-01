# EyeMotion Unified Build Script
# Usage: .\build_all.ps1

$ErrorActionPreference = "Stop"
$projectRoot = $PSScriptRoot

# 1. Setup Release Directory
$releaseDir = Join-Path $projectRoot "release"
if (Test-Path $releaseDir) {
    Write-Host "🧹 Cleaning up old release artifacts..." -ForegroundColor Yellow
    Remove-Item $releaseDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $releaseDir | Out-Null
Write-Host "📂 Release directory created: $releaseDir" -ForegroundColor Cyan

# Clean previous build artifacts
$oldExe = Join-Path $projectRoot "target\release\eyemotion.exe"
if (Test-Path $oldExe) { Remove-Item $oldExe -Force }

# 2. Build Frontend
Write-Host "🎨 Building Frontend..." -ForegroundColor Yellow
npm run build
if ($LASTEXITCODE -ne 0) { throw "Frontend build failed" }

# 3. Build Windows
Write-Host "🪟 Building Windows..." -ForegroundColor Yellow
npm run tauri build
if ($LASTEXITCODE -ne 0) { throw "Windows build failed" }

# Collect Windows Artifacts
$targetDir = Join-Path $projectRoot "target\release\bundle"
$winSetup = Get-ChildItem "$targetDir\nsis\*.exe" | Select-Object -First 1
$winMsi = Get-ChildItem "$targetDir\msi\*.msi" | Select-Object -First 1
$winExe = Join-Path $projectRoot "target\release\eyemotion.exe"

if ($winSetup) { Copy-Item $winSetup.FullName "$releaseDir\EyeMotion_Windows_Setup.exe" -Force }
if ($winMsi) { Copy-Item $winMsi.FullName "$releaseDir\EyeMotion_Windows_Installer.msi" -Force }

# Create Portable Green Version
if (Test-Path $winExe) {
    Write-Host "📦 Creating Portable Green Version..." -ForegroundColor Yellow
    $portableDir = Join-Path $releaseDir "EyeMotion_Portable_Green"
    New-Item -ItemType Directory -Force -Path $portableDir | Out-Null
    Copy-Item $winExe "$portableDir\EyeMotion.exe" -Force
    $loaderDll = Join-Path $projectRoot "target\release\WebView2Loader.dll"
    if (Test-Path $loaderDll) { Copy-Item $loaderDll "$portableDir\" -Force }
    Compress-Archive -Path "$portableDir\*" -DestinationPath "$releaseDir\EyeMotion_Portable_Green.zip" -Force
    Remove-Item $portableDir -Recurse -Force
}

# 4. Build Android
Write-Host "🤖 Building Android..." -ForegroundColor Yellow

# Load Env (Essential for Android Build)
. "$projectRoot\local_env.ps1"

# Verify Env
if (-not $env:JAVA_HOME) { throw "JAVA_HOME is not set" }
if (-not $env:ANDROID_HOME) { throw "ANDROID_HOME is not set" }

Write-Host "   Using JAVA_HOME: $env:JAVA_HOME" -ForegroundColor Gray
Write-Host "   Using ANDROID_HOME: $env:ANDROID_HOME" -ForegroundColor Gray

# Force Patch tauri.conf.json
Write-Host "   🔧 Patching tauri.conf.json to FORCE remove devUrl..." -ForegroundColor Cyan
node "$projectRoot\patch_config.js"

try {
    # Use Standard Tauri CLI for Android Build (but tolerate symlink errors)
    Write-Host "   🚀 Running Tauri Android Build (APK)..." -ForegroundColor Cyan

    $tauriBuildFailed = $false
    try {
        # We use 'npm run tauri -- android build' to strictly pass args
        # Wrap in try/catch block to prevent script termination on error
        cmd /c "npm run tauri -- android build"
        if ($LASTEXITCODE -ne 0) { 
            Write-Warning "Tauri CLI command returned non-zero exit code ($LASTEXITCODE)."
            $tauriBuildFailed = $true 
        }
    } catch {
        Write-Warning "Tauri CLI execution threw an exception: $_"
        $tauriBuildFailed = $true
    }

    # Always attempt recovery if .so exists
    $soPath = Join-Path $projectRoot "target\aarch64-linux-android\release\libeyemotion.so"
    if (Test-Path $soPath) {
        if ($tauriBuildFailed) {
             Write-Host "   ⚠️ Tauri CLI build failed (likely symlink error), but Rust lib exists." -ForegroundColor Yellow
             Write-Host "   🛠️ Proceeding with MANUAL copy and Gradle build..." -ForegroundColor Cyan
        } else {
             Write-Host "   ✅ Tauri CLI build reported success (or we are double-checking)." -ForegroundColor Green
        }

        # 1. Copy .so to jniLibs (Manual Fix for Symlink Error)
        $jniDir = Join-Path $projectRoot "src-tauri\gen\android\app\src\main\jniLibs\arm64-v8a"
        if (-not (Test-Path $jniDir)) { New-Item -ItemType Directory -Force -Path $jniDir | Out-Null }
        Copy-Item $soPath "$jniDir\libeyemotion.so" -Force
        Write-Host "      - Copied libeyemotion.so" -ForegroundColor Gray
        
        # 2. Copy Assets (dist -> assets) (Manual Fix for Assets)
        $assetsDir = Join-Path $projectRoot "src-tauri\gen\android\app\src\main\assets"
        # Ensure assets dir exists
        if (-not (Test-Path $assetsDir)) { New-Item -ItemType Directory -Force -Path $assetsDir | Out-Null }
        
        # Copy dist content
        $distDir = Join-Path $projectRoot "dist"
        Copy-Item "$distDir\*" $assetsDir -Recurse -Force
        Write-Host "      - Copied assets from dist" -ForegroundColor Gray
        
        # 3. Run Gradle (Manual Build)
        Write-Host "      - Running Gradle AssembleRelease..." -ForegroundColor Cyan
        Set-Location "$projectRoot\src-tauri\gen\android"
        
        # Use cmd /c to run gradlew.bat
        cmd /c "gradlew.bat assembleRelease"
        
        if ($LASTEXITCODE -ne 0) { throw "Gradle Build Failed" }
        
        # Return to root
        Set-Location $projectRoot
            
    } else {
        throw "Rust library compilation failed. .so file not found at $soPath"
    }

    # Collect Android Artifacts
    # Tauri v2 usually outputs to: src-tauri/gen/android/app/build/outputs/apk
    $apkDir = Join-Path $projectRoot "src-tauri\gen\android\app\build\outputs\apk"
    $apkDebug = "$apkDir\debug\app-debug.apk"
    $apkRelease = "$apkDir\release\app-release.apk"
    $apkReleaseUnsigned = "$apkDir\release\app-release-unsigned.apk"

    if (Test-Path $apkDebug) { Copy-Item $apkDebug "$releaseDir\EyeMotion_Android_Debug.apk" -Force }
    if (Test-Path $apkRelease) { 
        Copy-Item $apkRelease "$releaseDir\EyeMotion_Android_Release.apk" -Force 
    } elseif (Test-Path $apkReleaseUnsigned) {
        Copy-Item $apkReleaseUnsigned "$releaseDir\EyeMotion_Android_Release_Unsigned.apk" -Force
    }

} finally {
    # Restore tauri.conf.json
    Write-Host "   🔄 Restoring tauri.conf.json..." -ForegroundColor Cyan
    node "$projectRoot\restore_config.js"
    # Ensure we are back at root
    Set-Location $projectRoot
}

Write-Host "✅ All builds completed successfully!" -ForegroundColor Green
Write-Host "🚀 Artifacts are available in: $releaseDir" -ForegroundColor Green
Get-ChildItem $releaseDir | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
