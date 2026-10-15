@echo off
setlocal
powershell.exe -NoLogo -NoProfile -STA -ExecutionPolicy Bypass -File "%~dp0Start-Skybox-Mod.ps1"
set "skybox_exit=%errorlevel%"
if not "%skybox_exit%"=="0" pause
exit /b %skybox_exit%
