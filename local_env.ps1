$jdkPath = Get-ChildItem -Path "$PSScriptRoot\jdk-17" -Directory | Select-Object -First 1 -ExpandProperty FullName
$env:JAVA_HOME = $jdkPath
$env:Path = "$jdkPath\bin;$env:Path"

$env:ANDROID_HOME = "$PSScriptRoot\android_sdk"
$env:Path += ";$env:ANDROID_HOME\cmdline-tools\latest\bin;$env:ANDROID_HOME\platform-tools"

Write-Host "✅ Android Environment Configured!"
Write-Host "JAVA_HOME: $env:JAVA_HOME"
Write-Host "ANDROID_HOME: $env:ANDROID_HOME"
Write-Host "You can now run 'cargo tauri android build'"
