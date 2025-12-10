default:
	if ! command -v cargo >/dev/null 2>&1; then curl https://sh.rustup.rs -sSf | sh -s -- -y ; fi
	if [ ! -f /usr/include/ncurses.h ]; then sudo apt install libncurses-dev -y ; fi
	. "${HOME}/.cargo/env" && cargo run --release

tty1:
	. "${HOME}/.cargo/env" && cargo build --release
	sudo openvt -f -c 1 -s -w -- ./target/release/tac

clean:
	rm -rf target `find . -name \*~`
