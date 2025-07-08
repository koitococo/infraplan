ci-fmt:
	cargo fmt --all -- --check

ci-clippy:
	cargo clippy --all-targets --all-features -- -D warnings

ci-test:
	cargo test --all-targets --all-features

ci: ci-fmt ci-clippy ci-test

build:
	cargo build --bins -Z unstable-options --artifact-dir ./artifacts

fix:
	cargo clippy --fix --all-targets --all-features --allow-dirty --broken-code
	cargo fmt --all

commit-fix:
	git -C ./libparted add -A && \
	  git -C ./libparted diff-index --quiet HEAD || \
		git -C ./libparted commit -m 'style: apply `cargo fmt` and `cargo fix`'
	git -C ./tokio-tar add -A && \
	  git -C ./tokio-tar diff-index --quiet HEAD || \
		git -C ./tokio-tar commit -m 'style: apply `cargo fmt` and `cargo fix`'
	git add -A && \
	  git diff-index --quiet HEAD || \
		git commit -m 'style: apply `cargo fmt` and `cargo fix`'

git-push:
	git -C ./libparted push
	git -C ./tokio-tar push
	git push

test:
	cargo test --all-targets --all-features

push-remote: build
	scp ./artifacts/* vfedora:

debug-remote: test push-remote
	ssh -t vfedora "./infraplan -v apply deploy_ubuntu.yaml"
