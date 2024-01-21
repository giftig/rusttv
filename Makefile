.PHONY: dist

test:
	@cargo test

clean:
	@rm -Rf target dist

build:
	cargo build

build/win:
	cargo build --target x86_64-pc-windows-gnu

build/cross: build
build/cross: build/win

build/release:
	cargo build --release
	cargo build --release --target x86_64-pc-windows-gnu

dist:
	./scripts/dist.sh

dist/win:
	./scripts/dist.sh x86_64-pc-windows-gnu

dist/cross: dist
dist/cross: dist/win
