@echo off
setlocal

:: ================= 配置项目信息 =================
set APP_NAME=jiujie
set VERSION=0.0.1
set TARGET_ARCH=x86_64-pc-windows-msvc
set TARGET_DIR=target\%TARGET_ARCH%\release
set WIX_OBJ_DIR=target\wix
set MSI_NAME=%APP_NAME%-%VERSION%-x86_64.msi

:: ================= 检查 WiX 工具链 =================
:: 检测 candle.exe (编译器) 和 light.exe (链接器) 是否在 PATH 中
where /q candle.exe
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] WiX Toolset not found in PATH.
    echo.
    echo Please install WiX Toolset v3.11:
    echo https://github.com/wixtoolset/wix3/releases
    echo.
    echo Or if installed via cargo-wix, add its bin to PATH (usually %%USERPROFILE%%\.cargo\wix\bin)
    exit /b 1
)

:: ================= 第一步：编译 Rust 项目 =================
echo [1/4] Building Rust Project (Release)...
cargo build --release --target %TARGET_ARCH%
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Cargo build failed.
    exit /b %ERRORLEVEL%
)

:: ================= 第二步：准备构建目录 =================
echo [2/4] Preparing WiX directories...
if not exist "%WIX_OBJ_DIR%" mkdir "%WIX_OBJ_DIR%"

:: ================= 第三步：编译 WiX 源码 (Candle) =================
echo [3/4] Compiling installer definition...
:: -dCargoTargetBinDir: 传入 Rust 编译产物的路径，供 .wxs 文件引用
:: -arch x64: 指定架构
:: -ext WixUIExtension: 启用 UI 扩展库
candle.exe -dVersion=%VERSION% -dCargoTargetBinDir="%TARGET_DIR%" -arch x64 -ext WixUIExtension -out "%WIX_OBJ_DIR%\%APP_NAME%.wixobj" wix\%APP_NAME%.wxs
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] WiX compilation failed.
    exit /b %ERRORLEVEL%
)

:: ================= 第四步：链接生成 MSI (Light) =================
echo [4/4] Linking MSI package...
:: -cultures:zh-CN : 强制指定中文环境
:: -loc wix\main.wxl : 显式加载我们的本地化文件（解决 LGHT0102 的关键！)
:: -sice:ICE61 : 忽略某些不影响安装的升级检查警告
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
