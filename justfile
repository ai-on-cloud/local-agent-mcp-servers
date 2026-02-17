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

integration-test:
    cargo test -p mcp-browser-core --test code_mode_browser -- --ignored --test-threads=1

# Open browser for manual login, then save profile
setup-login profile url:
    cargo run -p browser-server -- setup-login --profile {{profile}} --url {{url}}

install:
    cargo install --path browser-server
