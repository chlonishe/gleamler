.PHONY: all gen test clean

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    LIB_EXT = so
else
    LIB_EXT = so
endif

all: gen
	cargo build -p gleamler --release
	mkdir -p priv
	cp target/release/libgleamler.$(LIB_EXT) priv/gleamler.$(LIB_EXT)
	gleam build

gen:
	cargo run -p gleamler_codegen -- gleamler/src/nifs.rs src/gleamler_nif_ffi.erl src/gleamler_nif.gleam

test: all
	gleam test

clean:
	rm -rf priv target build
	cargo clean