@echo off
echo Running next.pl test...
echo.

REM Check if perl is available
where perl >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Perl is not installed or not in PATH
    echo Please install Perl and ensure it's available in your system PATH
    pause
    exit /b 1
)

REM Change to the tests directory and run next.pl
cd /d "%~dp0tests"
if exist "next.pl" (
    echo Found next.pl, running test...
    perl next.pl
    echo.
    echo Test completed with exit code: %errorlevel%
) else (
    echo Error: next.pl not found in tests directory
    echo Current directory: %CD%
    echo Available files:
    dir /b
)

echo.
