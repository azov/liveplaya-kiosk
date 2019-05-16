#!/bin/bash 
DIST="2018-04-18-raspbian-stretch"

get_device_name() {
	sample=$1
	dd_flags=$2
	echo "Please find the right device, ${sample}."
	read -p "Enter device name to overwrite: " dev
	echo "PLEASE CHECK ONE MORE TIME, WRITING TO WRONG DEVICE WILL FUCK UP YOUR COMPUTER !!!"
	while [[ "${overwrite_device}" != "yes"  && "${overwrite_device}" != "skip" ]] ; do
		read -p "Do you want to overwrite ${dev}? You can type 'yes' or 'skip': " overwrite_device
	done
	if [[ "${overwrite_device}" != "yes" ]] ; then
		dev=""
	fi
}

enable_ssh() {
	sdcard_home=$1
	if ! touch ${sdcard_home}/ssh ; then
		exit -1
	fi
	echo "Created ${sdcard_home}/ssh"
}

setup_wifi() {
	sdcard_home=$1

	read -p "Enter SSID or press Enter to skip setting up WiFi: " ssid
	if [[ -z "${ssid}" ]]; then
		echo "Skipping WiFi setup..."
		return
	fi
	
	read -s -p "Enter WiFi password: " pass

	echo ""
	if ! printf "ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev\n\
network={\n\
	ssid="${ssid}"\n\
	psk="${pass}"\n\
	key_mgmt=WPA-PSK\n\
}\n" > ${sdcard_home}/wpa_supplicant.conf ; then

		exit -1
	fi
	echo "Wrote ${sdcard_home}/wpa_supplicant.conf"
	
}

if [[ "$OSTYPE" == "linux"* ]]; then
	echo "(we are on Linux)"
	SDCARD_HOME="/media/boot"
	DOWNLOADS="/tmp/"
	lsblk
	get_device_name "e.g. /dev/sdb or /dev/mmcblk0" 
	if [[ -z "${dev}" ]] ; then
		echo "Skipping writing image..."
	else
		echo ">" diskutil unmountDisk "${dev}"
		umount unmountDisk "${dev}"
	fi

	#"e.g. /dev/sdb1 or /dev/mmcblk0p1"
	# # Unmount all partitions: 
	# umount /dev/sdb1
	# umount /dev/sdb2
	# # ...
	# unzip -p 2018-04-18-raspbian-stretch.zip | sudo dd of=/dev/sdb1 bs=4M conv=fsync
	enable_ssh ${SDCARD_HOME}
	setup_wifi ${SDCARD_HOME}
	echo "Done"
	exit 0

elif [[ "$OSTYPE" == "darwin"* ]]; then
	echo "Here are the devices on your Mac:"
	SDCARD_HOME="/Volumes/boot"
	DOWNLOADS="~/Downloads"
	echo ">" diskutil list 
	diskutil list 
	get_device_name "e.g. /dev/disk3" 
	if [[ -z "${dev}" ]] ; then
		echo "Skipping writing image..."
	else
		echo ">" diskutil unmountDisk "${dev}"
		diskutil unmountDisk "${dev}"
	fi
	exit 0
		#sudo dd if=/Users/dmitrya/Downloads/2018-04-18-raspbian-stretch.img of=/dev/disk3 bs=4m

	if [[ -f "/Volumes/boot" ]]; then

		enable_ssh ${SDCARD_HOME}
		setup_wifi ${SDCARD_HOME}
		echo "Done"
		exit 0

	fi
	

else
	echo "This can only run on Linux or Mac"
	exit -1
fi
