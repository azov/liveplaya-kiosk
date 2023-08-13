.PHONY: all clean debug release

SOURCE_DIR := ${CURDIR}
BUILD_DIR := ${SOURCE_DIR}/build
MODULES_DIR := ${SOURCE_DIR}/node_modules

NPM = npm
BROWSERIFY = ${MODULES_DIR}/.bin/browserify
UGLIFY = ${MODULES_DIR}/.bin/uglifyjs

LIBRARY_SOURCES = $(shell find ${SOURCE_DIR} -type f -name "*.js" ! -path "${BUILD_DIR}/*" ! -path "${MODULES_DIR}/*")
LIBRARY_MAIN = ${SOURCE_DIR}/index.js 

all: debug release

# info: 
# 	@echo ${LIBRARY_SOURCES}

debug: $(SOURCE_DIR)/node_modules ${BUILD_DIR}/debug.js  

release: $(SOURCE_DIR)/node_modules ${BUILD_DIR}/min.js

clean:
	rm -rf ${BUILD_DIR}
	rm -rf ${MODULES_DIR}

$(SOURCE_DIR)/node_modules: 
	$(NPM) install 

${BUILD_DIR}/min.js: node_modules ${LIBRARY_SOURCES}
	@mkdir -p $(dir $@)
	NODE_ENV=production $(BROWSERIFY) --transform [ babelify --presets [ @babel/preset-env ] --plugins [ @babel/plugin-proposal-class-properties ] ] ${LIBRARY_MAIN} |\
		$(UGLIFY) --compress --mangle > $@.tmp && mv $@.tmp $@

${BUILD_DIR}/debug.js: node_modules ${LIBRARY_SOURCES} 
	@mkdir -p $(dir $@)
	NODE_ENV=development $(BROWSERIFY) --transform [ babelify --presets [ @babel/preset-env ] --plugins [ @babel/plugin-proposal-class-properties ] ] --debug ${LIBRARY_MAIN} > $@.tmp && mv $@.tmp $@
	

