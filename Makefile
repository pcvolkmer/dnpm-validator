ifndef VERBOSE
.SILENT:
endif

.PHONY: mingw64-binary-x86_64
mingw64-binary-x86_64:
	cd dnpm-validator-ui && mingw64-cmake -DCMAKE_BUILD_TYPE=Release && cd build/ && make

.PHONY: mingw64-package-x86_64
mingw64-package-x86_64: mingw64-binary-x86_64
	cd dnpm-validator-ui/build && cpack -G ZIP && cpack -G NSIS

.PHONY: linux-binary-x86_64
linux-binary-x86_64:
	cd dnpm-validator-ui && cmake -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build --config Release

.PHONY: linux-deb-x86_64
linux-deb-x86_64: linux-binary-x86_64
	cd dnpm-validator-ui/build && cpack -G DEB

.PHONY: linux-rpm-x86_64
linux-rpm-x86_64: linux-binary-x86_64
	cd dnpm-validator-ui/build && cpack -G RPM

.PHONY: clean
clean:
	cargo clean
	rm -rf dnpm-validator-ui/build-mingw64 &2>/dev/null || true
	rm -rf dnpm-validator-ui/build &2>/dev/null || true