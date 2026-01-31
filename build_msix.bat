@echo off
setlocal

:: ================= 配置 =================
set APP_NAME=jiujie
set TARGET_ARCH=x86_64-pc-windows-msvc
set TARGET_DIR=target\%TARGET_ARCH%\release
set MSIX_LAYOUT_DIR=target\msix_layout
set MSIX_NAME=%APP_NAME%-store-package.msix

:: ================= 检查环境 =================
:: 尝试查找 MakeAppx.exe (Windows SDK)
where /q MakeAppx.exe
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] MakeAppx.exe not found!
    echo Please ensure Windows SDK is installed and in your PATH.
    echo Likely path: C:\Program Files (x86)\Windows Kits\10\bin\10.0.xxxxx.0\x64
    echo You can run this script from "Developer Command Prompt for VS".
    exit /b 1
)

:: ================= 1. 编译 =================
echo [1/4] Building Release Binary...
cargo build --release --target %TARGET_ARCH%
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%

:: ================= 2. 准备布局目录 =================
echo [2/4] Preparing Layout...
if exist "%MSIX_LAYOUT_DIR%" rmdir /s /q "%MSIX_LAYOUT_DIR%"
mkdir "%MSIX_LAYOUT_DIR%"
mkdir "%MSIX_LAYOUT_DIR%\Assets"

:: 复制主程序
copy "%TARGET_DIR%\%APP_NAME%.exe" "%MSIX_LAYOUT_DIR%\"

:: 复制 assets 文件夹
xcopy /E /I /Y "assets" "%MSIX_LAYOUT_DIR%\assets"

:: 复制 Manifest
copy "store\AppxManifest.xml" "%MSIX_LAYOUT_DIR%\"

:: 复制并重命名图标以满足商店要求 (临时使用 icon_256.png)
echo [INFO] Generating placeholder store icons from assets...
copy "assets\icons\icon_256.png" "%MSIX_LAYOUT_DIR%\Assets\StoreLogo.png"
copy "assets\icons\icon_256.png" "%MSIX_LAYOUT_DIR%\Assets\Square150x150Logo.png"
copy "assets\icons\icon_256.png" "%MSIX_LAYOUT_DIR%\Assets\Square44x44Logo.png"

:: ================= 3. 打包 MSIX =================
echo [3/4] Packing MSIX...
if exist "%MSIX_NAME%" del "%MSIX_NAME%"

MakeAppx pack /d "%MSIX_LAYOUT_DIR%" /p "%MSIX_NAME%"
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Packing failed!
    exit /b %ERRORLEVEL%
)

echo.
echo ==========================================
echo SUCCESS! Store package generated:
echo %CD%\%MSIX_NAME%
echo ==========================================
echo.
echo [NEXT STEPS]
echo 1. Test installation (requires certificate):
    Add-AppxPackage %MSIX_NAME%
echo.
    
echo 2. For Microsoft Store upload:
    echo    The Store will sign it automatically. You can upload this .msix file.
echo ==========================================
pause
