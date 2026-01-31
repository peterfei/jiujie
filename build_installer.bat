@echo off
setlocal

:: ================= 配置项目信息 =================
set APP_NAME=jiujie
set VERSION=0.0.1
set TARGET_ARCH=x86_64-pc-windows-msvc
set TARGET_DIR=target\%TARGET_ARCH%\release
set WIX_OBJ_DIR=target\wix
set MSI_NAME=%APP_NAME%-%VERSION%-x86_64.msi

echo [INFO] Project: %APP_NAME%
echo [INFO] Version: %VERSION%
echo [INFO] MSI Output: %MSI_NAME%

:: ================= 检查并设置环境变量 =================
echo [INFO] Checking WiX Toolset...
:: 尝试自动添加 cargo-wix 可能的安装路径
set "PATH=%PATH%;%USERPROFILE%\.cargo\wix\bin"

where /q candle.exe
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] candle.exe not found!
    echo [TIP] Please ensure WiX Toolset is in your PATH.
    echo [TIP] Default cargo-wix path: %USERPROFILE%\.cargo\wix\bin
    pause
    exit /b 1
)
echo [INFO] WiX Toolset found.

:: ================= 第一步：编译 Rust 项目 =================
echo.

echo [1/4] Building Rust Project (Release)...
cargo build --release --target %TARGET_ARCH%
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Cargo build failed!
    pause
    exit /b %ERRORLEVEL%
)

:: ================= 第二步：准备构建目录 =================
echo.

echo [2/4] Preparing WiX directories...
if not exist "%WIX_OBJ_DIR%" mkdir "%WIX_OBJ_DIR%"
if exist "%MSI_NAME%" del "%MSI_NAME%"

:: ================= 第三步：编译 WiX 源码 (Candle) =================
echo.

echo [3/4] Compiling installer definition...
echo [CMD] candle.exe -dVersion="%VERSION%" -dCargoTargetBinDir="%TARGET_DIR%" -arch x64 -ext WixUIExtension -out "%WIX_OBJ_DIR%\%APP_NAME%.wixobj" wix\%APP_NAME%.wxs

candle.exe -dVersion="%VERSION%" -dCargoTargetBinDir="%TARGET_DIR%" -arch x64 -ext WixUIExtension -out "%WIX_OBJ_DIR%\%APP_NAME%.wixobj" wix\%APP_NAME%.wxs

if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] WiX compilation failed!
    pause
    exit /b %ERRORLEVEL%
)

:: ================= 第四步：链接生成 MSI (Light) =================
echo.

echo [4/4] Linking MSI package...
echo [CMD] light.exe -ext WixUIExtension -cultures:zh-CN -loc wix\main.wxl -out "%MSI_NAME%" "%WIX_OBJ_DIR%\%APP_NAME%.wixobj" -sice:ICE61

light.exe -ext WixUIExtension -cultures:zh-CN -loc wix\main.wxl -out "%MSI_NAME%" "%WIX_OBJ_DIR%\%APP_NAME%.wixobj" -sice:ICE61

if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] WiX linking failed!
    pause
    exit /b %ERRORLEVEL%
)

:: ================= 验证结果 =================
echo.
if exist "%MSI_NAME%" (
    echo ==========================================
    echo  SUCCESS! Installer generated at:
    echo  %CD%\%MSI_NAME%
    echo ==========================================
) else (
    echo [ERROR] Build finished but MSI file is missing!
)

pause
