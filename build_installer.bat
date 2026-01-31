@echo off
setlocal

:: ================= 配置项目信息 =================
set APP_NAME=jiujie
set VERSION=0.0.1
set TARGET_ARCH=x86_64-pc-windows-msvc
set TARGET_DIR=target\%TARGET_ARCH%\release
set WIX_OBJ_DIR=target\wix
set MSI_NAME=%APP_NAME%-%VERSION%-x86_64.msi

:: ================= 检查并设置环境变量 =================
:: 尝试自动添加 cargo-wix 可能的安装路径
set PATH=%PATH%;%USERPROFILE%\.cargo\wix\bin

where /q candle.exe
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] candle.exe not found. 
    echo Please ensure WiX Toolset is in PATH.
    echo Default cargo-wix path: %%USERPROFILE%%\.cargo\wix\bin
    exit /b 1
)

:: ================= 第一步：编译 Rust 项目 =================
echo [1/4] Building Rust Project (Release)...
call cargo build --release --target %TARGET_ARCH%
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Cargo build failed.
    exit /b %ERRORLEVEL%
)

:: ================= 第二步：准备构建目录 =================
echo [2/4] Preparing WiX directories...
if not exist "%WIX_OBJ_DIR%" mkdir "%WIX_OBJ_DIR%"

:: ================= 第三步：编译 WiX 源码 (Candle) =================
echo [3/4] Compiling installer definition...
candle.exe -dVersion="%VERSION%" -dCargoTargetBinDir="%TARGET_DIR%" -arch x64 -ext WixUIExtension -out "%WIX_OBJ_DIR%\%APP_NAME%.wixobj" wix\%APP_NAME%.wxs
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] WiX compilation failed.
    exit /b %ERRORLEVEL%
)

:: ================= 第四步：链接生成 MSI (Light) =================
echo [4/4] Linking MSI package...
light.exe -ext WixUIExtension -cultures:zh-CN -loc wix\main.wxl -out "%MSI_NAME%" "%WIX_OBJ_DIR%\%APP_NAME%.wixobj" -sice:ICE61
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] WiX linking failed.
    exit /b %ERRORLEVEL%
)

echo.
echo ==========================================
echo  SUCCESS! Installer generated:
echo  %CD%\%MSI_NAME%
echo ==========================================
pause