Local build: 

```shell
    cargo build
    cargo run -- --help 
```
(then follow the commands)


Docker build (this should cross-compile for Raspberry PI):

```shell


```





### Building from skratch on Raspberry Pi

Install Raspbian official repo and ssh to it. Call it tgwtf. Then
ssh tgwtf@tgwtf.local and type: 

```
    # Dev tools
    sudo apt-get update 
    sudo apt-get install -y\
	    apt-utils\
	    git\
	    build-essential\
	    curl\
		clang\
		libclang-dev

    # Node
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - &&\
        sudo apt-get install -y nodejs
	
    # Rust
    curl https://sh.rustup.rs -sSf | sh -s -- -y
    source "$HOME/.cargo/env"

    # Sources
	git clone https://github.com/azov/liveplaya-kiosk.git
	cd liveplaya-kiosk
    
    # This will take a while
    cargo build 

    # 
```
