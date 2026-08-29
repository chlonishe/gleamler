.PHONY: all test clean leak-test valgrind-test

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
	cargo run -p gleamler_codegen -- gleamler src/gleamler_nif_ffi.erl src/gleamler_nif.gleam --with-stress
	gleam build

test: all
	gleam test

leak-test: all
	erlc -o build/dev/erlang/gleamler/ebin src/resource_leak_test.erl
	erl +S 1:1 -noshell -pa build/dev/erlang/gleamler/ebin -s resource_leak_test run -s init stop

valgrind-test: all
	valgrind --leak-check=full --show-leak-kinds=definite,indirect \
		--errors-for-leak-kinds=definite,indirect \
		--error-exitcode=1 \
		--suppressions=valgrind.supp \
		erl +S 1:1 -noshell -pa $(shell find build/dev/erlang -name ebin) \
		-s resource_leak_test run -s init stop

clean:
	rm -rf priv target build
	cargo clean
