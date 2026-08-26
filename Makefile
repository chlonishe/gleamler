.PHONY: all test clean

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    SRC_LIB = target/release/libgleamler.dylib
else
    SRC_LIB = target/release/libgleamler.so
endif

all:
	cargo build -p gleamler --release --features stress
	mkdir -p priv
	cp $(SRC_LIB) priv/gleamler.so
	cargo run -p gleamler_codegen -- gleamler src/gleamler_nif_ffi.erl src/gleamler_nif.gleam
	gleam build

test: all
	gleam test

clean:
	rm -rf priv target build
	cargo clean