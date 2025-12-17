bindings: src/bindings

src/bindings: weiroll/node_modules
	forge bind \
		--hardhat \
		--root weiroll \
		--module \
		--bindings-path ./src/bindings \
		--select-all

submodules: weiroll/.git

weiroll/.git:
	git submodule update --init --recursive weiroll

weiroll/node_modules: weiroll/.git
	cd weiroll && npm install

clean:
	rm -rf ./src/bindings

.PHONY: bindings clean rustfmt submodules