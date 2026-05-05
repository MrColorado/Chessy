VERSION ?= dev

.PHONY: all uci lichess install clean

all: uci lichess

uci:
	VERSION=$(VERSION) cargo build --release --bin chess-uci

lichess:
	VERSION=$(VERSION) cargo build --release --bin chess-lichess

install: all
	cp target/release/chess-uci     target/release/chess-uci-$(VERSION)
	cp target/release/chess-lichess  target/release/chess-lichess-$(VERSION)

clean:
	cargo clean
