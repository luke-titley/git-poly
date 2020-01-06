from centos:6
# Install cargo
run curl --tlsv1.2 -sSf https://sh.rustup.rs > ./install_rustup.sh
run echo '' | bash ./install_rustup.sh -y
env PATH "/root/.cargo/bin:${PATH}"
# Install g++
run yum install -y gcc
run rustup target add x86_64-unknown-linux-musl
# Build the project
add src src
add Cargo.toml Cargo.toml
run cargo build --release --target x86_64-unknown-linux-musl
