default:
	if ! command -v cargo >/dev/null 2>&1; then curl https://sh.rustup.rs -sSf | sh -s -- -y ; fi
	if [ ! -f /usr/include/ncurses.h ]; then sudo apt install libncurses-dev -y ; fi
	. "${HOME}/.cargo/env" && cargo run --release
clean:
	rm -rf target `find . -name \*~`
