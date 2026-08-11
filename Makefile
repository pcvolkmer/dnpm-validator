ifndef VERBOSE
.SILENT:
endif


.PHONY: win-binary-x86_64
win-binary-x86_64:
	cargo build -r --workspace --target x86_64-pc-windows-gnu
	mkdir dnpm-validator-ui/libs &2>/dev/null || true
	mkdir dnpm-validator-ui/build &2>/dev/null || true
	cp target/x86_64-pc-windows-gnu/release/libdnpmvalidation.a dnpm-validator-ui/libs/ &2>/dev/null || true
	cp target/x86_64-pc-windows-gnu/cxxbridge/dnpmvalidation/src/* dnpm-validator-ui/libs/ &2>/dev/null || true
	cp target/x86_64-pc-windows-gnu/cxxbridge/rust/cxx.h dnpm-validator-ui/libs/ &2>/dev/null || true
	cd dnpm-validator-ui && mingw64-cmake -DCMAKE_BUILD_TYPE=Release ./build && cd build/ && make

.PHONY: win-package-x86_64
win-package-x86_64: win-binary-x86_64
	strip dnpm-validator-ui/build/dnpm-validator-ui.exe
	cd dnpm-validator-ui/build && cpack -G ZIP && cpack -G NSIS

.PHONY: linux-binary-x86_64
linux-binary-x86_64:
	cargo build -r --workspace --target x86_64-unknown-linux-gnu
	mkdir dnpm-validator-ui/libs &2>/dev/null || true
	cp target/x86_64-unknown-linux-gnu/release/libdnpmvalidation.a dnpm-validator-ui/libs/ &2>/dev/null || true
	cp target/x86_64-unknown-linux-gnu/cxxbridge/dnpmvalidation/src/* dnpm-validator-ui/libs/ &2>/dev/null || true
	cp target/x86_64-unknown-linux-gnu/cxxbridge/rust/cxx.h dnpm-validator-ui/libs/ &2>/dev/null || true
	cd dnpm-validator-ui && cmake -B build && cd build/ && make

.PHONY: clean
clean:
	cargo clean
	rm -rf dnpm-validator-ui/build