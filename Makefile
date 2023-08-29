.PHONY: all debug release-armv7 run clean deploy
SOURCE_DIR:=${CURDIR}
TARGET_DIR:=${CURDIR}/target
CARGO:=cargo
DOCKER:=docker
IMAGE:=tgwtf

all: docker-build


docker-build:
	mkdir -p ${TARGET_DIR}
	${DOCKER} build -t ${IMAGE} ${SOURCE_DIR}
	${DOCKER} run -v ${SOURCE_DIR}:/src -v ${TARGET_DIR}:/build ${IMAGE} release-armv7

docker-run: docker-build
	${DOCKER} run -it --rm -p 8080:8080 tgwtf

docker-deploy: docker-build
	@if [ -z ${TGWTF_HOST} ]; then echo "Please set TGWTF_HOST environment variable" && exit 255; fi
	${DOCKER} run -it --rm  -e "TGWTF_HOST=${TGWTF_HOST}" tgwtf deploy

docker-shell: 
	${DOCKER} run -it --rm -v${SOURCE_DIR}:/workdir --entrypoint=/bin/bash tgwtf

release-armv7:
	${CARGO} build --target=armv7-unknown-linux-gnueabihf --release

clean: 
	${CARGO} clean

setup: 
	@if [ -z ${TGWTF_HOST} ]; then echo "Please set TGWTF_HOST environment variable" && exit -1; fi
	cat ${HOME}/.ssh/id_rsa.pub | ssh pi@${TGWTF_HOST} 'sudo mount -o remount,rw / && mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys'
	ssh pi@${TGWTF_HOST} 'sudo mkdir -p /root/.ssh && sudo cp /home/pi/.ssh/authorized_keys /root/.ssh/authorized_keys'

deploy: release-armv7
	@if [ -z ${TGWTF_HOST} ]; then echo "Please set TGWTF_HOST environment variable" && exit -1; fi
	@if [ -z ${CARGO_TARGET_DIR} ]; then echo "Please set CARGO_TARGET_DIR environment variable" && exit -1; fi
	-ssh root@${TGWTF_HOST} "service tgwtf stop"
	ssh pi@${TGWTF_HOST} "sudo mount -o remount,rw /"
	ssh root@${TGWTF_HOST} "mkdir -p /opt/tgwtf/bin && mkdir -p /opt/tgwtf/etc && mkdir -p /opt/tgwtf/www"	
	scp ${SOURCE_DIR}/etc/tgwtf.service.conf root@${TGWTF_HOST}:/etc/systemd/system/tgwtf.service
	scp ${SOURCE_DIR}/www/index.html root@${TGWTF_HOST}:/opt/tgwtf/www/index.html
	scp ${SOURCE_DIR}/www/liveplaya.debug.js root@${TGWTF_HOST}:/opt/tgwtf/www/liveplaya.debug.js
	scp ${CARGO_TARGET_DIR}/armv7-unknown-linux-gnueabihf/release/tgwtf root@${TGWTF_HOST}:/opt/tgwtf/bin/tgwtf
	ssh -t root@${TGWTF_HOST} "systemctl enable tgwtf && systemctl reset-failed tgwtf && systemctl start tgwtf"
	ssh root@${TGWTF_HOST} "service tgwtf restart"
	-ssh root@${TGWTF_HOST} "sudo reboot"

