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
set "PATH=%PATH%;%USERPROFILE%\.cargo\wix\bin"

where /q candle.exe
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] candle.exe not found!
    exit /b 1
)
:: 检查 heat.exe (用于自动收集文件)
where /q heat.exe
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] heat.exe not found!
    exit /b 1
)
echo [INFO] WiX Toolset found.

:: ================= 第一步：编译 Rust 项目 =================
echo.
echo [1/5] Building Rust Project (Release)...
cargo build --release --target %TARGET_ARCH%
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Cargo build failed!
    pause
    exit /b %ERRORLEVEL%
)

:: ================= 第二步：准备构建目录 =================
echo.
echo [2/5] Preparing WiX directories...
if not exist "%WIX_OBJ_DIR%" mkdir "%WIX_OBJ_DIR%"
if exist "%MSI_NAME%" del "%MSI_NAME%"

:: ================= 第三步：收集资源文件 (Heat) =================
echo.
echo [3/5] Harvesting assets folder...
:: -dr: 指定目标安装目录 ID (APPLICATIONFOLDER)
:: -cg: 定义组件组 ID (AssetsComponentGroup)
:: -var: 定义预处理器变量 (var.AssetsDir)
:: -srd: 禁止收集根目录本身，只收集内容
:: -gg: 自动生成 GUID
heat.exe dir "assets" -dr APPLICATIONFOLDER -cg AssetsComponentGroup -var var.AssetsDir -gg -ke -srd -sfrag -template fragment -out "%WIX_OBJ_DIR%\assets.wxs"
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Assets harvesting failed!
    pause
    exit /b %ERRORLEVEL%
)

:: ================= 第四步：编译 WiX 源码 (Candle) =================
echo.
echo [4/5] Compiling installer definition...
:: 编译主文件
candle.exe -dVersion="%VERSION%" -dCargoTargetBinDir="%TARGET_DIR%" -dAssetsDir="assets" -arch x64 -ext WixUIExtension -out "%WIX_OBJ_DIR%\%APP_NAME%.wixobj" wix\%APP_NAME%.wxs
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%

:: 编译自动生成的资源文件
candle.exe -dVersion="%VERSION%" -dCargoTargetBinDir="%TARGET_DIR%" -dAssetsDir="assets" -arch x64 -ext WixUIExtension -out "%WIX_OBJ_DIR%\assets.wixobj" "%WIX_OBJ_DIR%\assets.wxs"
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%


:: ================= 第五步：链接生成 MSI (Light) =================
echo.
echo [5/5] Linking MSI package...
:: 注意：链接时需要包含两个 .wixobj 文件
light.exe -ext WixUIExtension -cultures:zh-CN -loc wix\main.wxl -out "%MSI_NAME%" "%WIX_OBJ_DIR%\%APP_NAME%.wixobj" "%WIX_OBJ_DIR%\assets.wixobj" -sice:ICE61
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