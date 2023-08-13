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

# Add deployment keys and configure ssh
ADD etc/tgwtf_deploy_id_rsa /root/.ssh/id_rsa
ADD etc/tgwtf_deploy_id_rsa.pub /root/.ssh/id_rsa.pub
RUN chmod 700 /root/.ssh &&\
	chmod 700 /root/.ssh/id_rsa &&\
	printf "    IdentityFile ~/.ssh/id_rsa\n    StrictHostKeyChecking no\n" >> /etc/ssh/ssh_config

# Download and compile tgwtf
RUN mkdir /src && \
	git clone git@gitlab.com:technogecko/tgwtf.git /src/tgwtf
WORKDIR /src/tgwtf
ENV CARGO_TARGET_DIR=/build/tgwtf
RUN make all

# Set up entry point
ADD . /workdir
WORKDIR /workdir
ENTRYPOINT ["/usr/bin/make"]
CMD ["run"]

