.PHONY: all test clean leak-test valgrind-test ci watch

all:
	cargo xtask build --release --stress

watch:
	cargo xtask watch

test: all
	cargo xtask test --gleam

leak-test: all
	cargo xtask test --leak

valgrind-test: all
	cargo xtask test --valgrind

clean:
	cargo xtask clean

ci:
	cargo xtask ci
