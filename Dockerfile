FROM ubuntu:18.04

# Set up development environment
RUN apt-get update && apt-get install -y\
	apt-utils\
	git\
	build-essential\
	curl

# Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Raspberry Pi toolchain
RUN apt-get install -y\
		clang\
		libclang-dev\
		gcc-arm-linux-gnueabihf\
		g++-arm-linux-gnueabihf &&\
	/root/.cargo/bin/rustup target add armv7-unknown-linux-gnueabihf
RUN printf "\n[target.armv7-unknown-linux-gnueabihf]\nlinker = \"arm-linux-gnueabihf-g++\"\nar = \"arm-linux-gnueabihf-ar\"\n" >> /root/.cargo/config

# Workaround a bug in bindgen (https://github.com/rust-lang-nursery/rust-bindgen/issues/1229)
ADD etc/x86_64-linux-gnu-includes /usr/include/x86_64-linux-gnu

# Common env
ENV PATH="/root/.cargo/bin:${PATH}"
ENV USER="root"

WORKDIR /src

ENTRYPOINT ["/usr/bin/make"]
#CMD ["run"]

