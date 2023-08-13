WTF Kiosk Setup
================

Before installation upload deployment keys (etc/tgwtf_deploy_id_rsa & etc/tgwtf_deploy_id_rsa.pub) on the 
target system and configure dfaprs as described in the corresponding subdir

Installation 
-------------
Serial configuration with the daemon managed by upstartd:

    sudo apt-get install -y git
    sudo apt-get install -y make
    sudo apt-get install -y nginx

    # why do we need node?!
	#curl -sL https://deb.nodesource.com/setup_6.x | sudo -E bash -
	#sudo apt-get install -y nodejs

    git clone https://gitlab.com/technogecko/tgwtf
    cd tgwtf
    make 
    sudo make install
 