rustup default  stable-x86_64-pc-windows-msvc
setlocal
call "%ProgramFiles%\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat"

set "target_dir=%~dp0examples"

for /d %%D in ("%target_dir%\*") do (
    if exist "%%D\Cargo.toml" (
        echo Running cargo build in %%~nxD
        pushd "%%D"
        call cargo build --release
        if errorlevel 1 (
            echo Build failed in %%~nxD
            popd
            exit /b 1
        )
        popd
    ) else (
        echo No Cargo.toml found in %%~nxD
    )
)

call powershell -c "[console]::beep(1000, 250)"
