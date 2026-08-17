ifndef VERBOSE
.SILENT:
endif


.PHONY: win-binary-x86_64
win-binary-x86_64:
	cd dnpm-validator-ui && mingw64-cmake -DCMAKE_BUILD_TYPE=Release && cd build/ && make

.PHONY: win-package-x86_64
win-package-x86_64: win-binary-x86_64
	cd dnpm-validator-ui/build && cpack -G ZIP && cpack -G NSIS

.PHONY: linux-binary-x86_64
linux-binary-x86_64:
	cd dnpm-validator-ui && cmake -B build -DCMAKE_BUILD_TYPE=Release && cd build/ && make

.PHONY: linux-package-x86_64
linux-package-x86_64: linux-binary-x86_64
	cd dnpm-validator-ui/build && cpack -G DEB && cpack -G RPM

.PHONY: clean
clean:
	cargo clean
	rm -rf dnpm-validator-ui/build-mingw64 &2>/dev/null || true
	rm -rf dnpm-validator-ui/build &2>/dev/null || true