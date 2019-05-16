.PHONY: all clean
SOURCE_DIR := ${CURDIR}
BUILD_DIR := ${SOURCE_DIR}/build

all:
	@echo "Building ${BUILD_DIR}..."

clean: 
	rm -rf ${BUILD_DIR}