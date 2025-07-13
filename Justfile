set dotenv-load
ROOT_PATH := justfile_directory()

fmt-check:
  cargo fmt --all -- --check

clippy:
  cargo clippy --all-targets --all-features -- -D warnings

test:
  cargo test --all-targets --all-features

build:
  cargo build --bins -Z unstable-options --artifact-dir ./artifacts

ci: fmt-check clippy test build

fix:
  cargo clippy --fix --all-targets --all-features --allow-dirty --broken-code
  cargo fmt --all

commit-fixs:
  git -C ./tokio-tar add -u && \
    git -C ./tokio-tar diff-index --quiet HEAD || \
    git -C ./tokio-tar commit -m 'style: apply `cargo fmt` and `cargo fix`'
  git add -u && \
    git diff-index --quiet HEAD || \
    git commit -m 'style: apply `cargo fmt` and `cargo fix`'

git-push:
  git -C ./tokio-tar push
  git push
