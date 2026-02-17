default: check test build

check:
    cargo fmt --check
    cargo clippy -- -D warnings

test:
    cargo test

build:
    cargo build --release

run port="3100" profile="":
    cargo run -p browser-server -- serve --port {{port}} --headless \
        {{ if profile != "" { "--profile " + profile } else { "" } }}

# Open browser for manual login, then save profile
setup-login profile url:
    cargo run -p browser-server -- setup-login --profile {{profile}} --url {{url}}
