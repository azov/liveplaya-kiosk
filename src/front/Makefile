.PHONY: all build dev clean
PROJECT_DIR:=${CURDIR}
NODE_MODULES=${PROJECT_DIR}/node_modules
NPM:=npm
DOCKER:=docker
WASM_PACK:=wasm_pack
TRUNK:=~/.cargo/bin/trunk 
CARGO:=~/.cargo/bin/cargo 

all: build

dev: ${NODE_MODULES}
	${NPM} run dev

build: ${NODE_MODULES}
	${NPM} run build

clean: 
	@rm -rf node_modules

${NODE_MODULES}:
	cd ${PROJECT_DIR} && ${NPM} install