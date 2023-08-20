# LivePlaya/WTF Kiosk

## Components

- Raspberry Pi 3B+
- 7" Raspberry Pi Touch Display: https://www.raspberrypi.org/products/raspberry-pi-touch-display/
- SmartPi Touch case: https://smarticase.com/collections/all/products/smartipi-touch

## Creating SD card

Download Raspberry Pi Imager (https://www.raspberrypi.com/software/)
Choose "Special OS"?? -> FullPageOS
Click the gear icon
Set hostname to tgwtf
Enable SSH
Use password authentication

Username/pass: tgwtf/****

Set locale settings

Disable telemetry

>>>>




Download the [latest Raspbian Stretch Lite distro](https://downloads.raspberrypi.org/raspbian_lite_latest) 
(this steps were tested with 2018-11-13 version].

Insert an empty SD card and run the following commands.

NOTE: BE CAREFUL WHEN LOOKING UP DEVICE NAME, WRITING TO A WRONG DEVICE MAY SCREW UP YOUR SYSTEM!

On Linux:

```shell
	lsblk
	# Find the right dev number, e.g. /dev/sdb (or /dev/mmcblk0) with partitions /dev/sdb1 (/dev/mmcblk0p1)
	# Unmount all partitions: 
	umount /dev/sdb1
	umount /dev/sdb2
	# ...

	export SDCARD_HOME=/media/boot
	unzip -p 2018-11-13-raspbian-stretch-lite.zip | sudo dd of=/dev/sdX1 bs=4M conv=fsync

	# We'll need this later

	#
	export TAR=tar
	#Enable ssh
	touch /media/boot/ssh
	cp etc/wpa_supplicant.conf /media/boot/
```

On Mac:

```shell
	diskutil list 
	# Find the right dev number, e.g. /dev/disk3

	diskutil unmountDisk /dev/disk3
	export SDCARD_HOME=/Volumes/boot
	unzip -p ~/Downloads/2018-11-13-raspbian-stretch-lite.zip | sudo dd of=/dev/disk3 bs=4m
```

Enable SSH:

```shell
	touch ${SDCARD_HOME}/ssh
```

Enable WiFi (if you have another way to connect to your Raspberry Pi, e.g. with Ethernet, you can skip this):

```shell
	printf "ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev\n\
	country=us\n\
	network={\n\
		ssid=\"${WIFI_SSID}\"\n\
		psk=\"${WIFI_PASS}\"\n\
		key_mgmt=WPA-PSK\n\
	}\n" > ${SDCARD_HOME}/wpa_supplicant.conf
```

Now you can eject the SD card, insert it into your Pi, and boot.

## Configuring Raspberry Pi

Connect to your Raspberry Pi box and run the following commands:

```shell
	# Connect to Raspberry Pi (the default password is 'paspberry')
	ssh -o "StrictHostKeyChecking no" pi@raspberrypi.local
	sudo su
	# You should see 'pi@raspberrypi:~ #' prompt 

	# Install X11 openbox and chromium
	apt update
	apt -y upgrade
	apt-get install -y --no-install-recommends xserver-xorg x11-xserver-utils xinit openbox chromium-browser

	# ------- continue from here -----





	# Enable autologin
	mkdir -p /etc/systemd/system/getty@tty1.service.d/
	print "[Service]\n\
		ExecStart=\n\
		ExecStart=-/sbin/agetty --autologin yourname --noclear %I $TERM\n\
	" > /etc/systemd/system/getty@tty1.service.d/override.conf	






	# Set up passwordless login. The default password is 'raspberry'
	cat ssh -o "StrictHostKeyChecking no" pi@raspberrypi.local -t "mkdir ~/.ssh" 
	
	scp pi@raspberrypi.local etc/authorized_keys pi@raspberrypi.local:.ssh/
	#from now on you shouldn't need the password
	scp pi@raspberrypi.local etc/id_rsa* pi@raspberrypi.local:.ssh/
	ssh pi@raspberrypi.local "sudo mkdir -p /root/.ssh && sudo cp ~/.ssh/authorized_keys /root/.ssh"

	# Rename it to tglight01
	#### NO, THIS DOESNT WORK!!!! ssh root@raspberrypi "echo "tglight01" > /etc/host && sed -E 's/raspberrypi/tglight01/' /etc/hosts > /etc/hosts.new && rm /etc/hosts && rm /etc/hosts.new /etc/hosts"

	ssh root@raspberrypi -t "mkdir -p /etc/X11/xorg.conf.d/" 
	scp etc/screencalibration.conf root@raspberrypi:/etc/X11/xorg.conf.d/99-calibration.conf

	# Change pi user password to something else so that it does not complain it's unsecure
	scp etc/shadow root@raspberrypi:/etc/shadow

	# Set up unclutter
	ssh -t pi@raspberrypi.local "sudo apt-get install -y unclutter"
	ssh -t pi@raspberrypi.local "mkdir -p ~/.config/lxsession/LXDE-pi/"
	scp etc/autostart pi@raspberrypi.local:~/.config/lxsession/LXDE-pi/autostart

	# Set up autorun script on a USB stick
	ssh -t pi@raspberrypi.local "echo \"@reboot /bin/bash /media/pi/AUTORUN/autorun.sh\" | crontab -"

```

