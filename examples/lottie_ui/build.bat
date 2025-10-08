rustup default  stable-x86_64-pc-windows-msvc
REM You need Visual Studio with C++ compiler and Windows SDK components
REM aws-lc-sys needs perl to compile openssl-sys as vendored
REM perl Strawberry will install nasm
call "%ProgramFiles%\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat"
REM here clang is installed from llvm.org
REM set PATH=%PATH%;%ProgramFiles%\LLVM\bin

REM cargo build --release --examples
REM cargo run --release --example renderlayers_performance
cargo build --release 
REM cargo build --release --example rxqlite-client
REM cargo build --release --example rxqlite-client-insecure-tls

REM rustup default  stable-x86_64-pc-windows-gnu

powershell -c "[console]::beep(1000, 250)"
