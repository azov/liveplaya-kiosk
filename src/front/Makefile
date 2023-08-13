.PHONY: all build dev clean
PROJECT_DIR:=${CURDIR}
NODE_MODULES=${PROJECT_DIR}/node_modules
NPM:=npm

all: build

dev: ${NODE_MODULES}
	${NPM} run dev

build: ${NODE_MODULES}
	${NPM} run build

clean: 
	@rm -rf node_modules

${NODE_MODULES}:
	cd ${PROJECT_DIR} && ${NPM} install