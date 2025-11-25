default:
	if ! command -v cargo >/dev/null 2>&1; then curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh ; fi
	if [ ! -f /usr/include/ncurses.h ]; then sudo apt install libncurses-dev -y ; fi
	cargo run --release
clean:
	rm -rf target `find . -name \*~`
