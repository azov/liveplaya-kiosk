# OS Setup for TechnoGecko Raspberry Pi's 

## Creating SD card 

Download 2018-04-18-raspbian-stretch.zip distro.

On Linux:

```shell
	lsblk
	# Find the right dev number, e.g. /dev/sdb (or /dev/mmcblk0) with partitions /dev/sdb1 (/dev/mmcblk0p1)
	# Unmount all partitions: 
	umount /dev/sdb1
	umount /dev/sdb2
	# ...
	unzip -p 2018-04-18-raspbian-stretch.zip | sudo dd of=/dev/sdX1 bs=4M conv=fsync
	# Enable ssh
	touch /media/boot/ssh
	cp etc/wpa_supplicant.conf /media/boot/
	export TAR=tar
```

Edit `wpa_supplicant.conf` and replace ${SSID} & ${PASS} with SSID & password.
Eject the SD card, insert it into Raspberry Pi, and boot.

After it boots :

```shell
	# the default password is 'paspberry'
	ssh pi@raspberrypi -t "mkdir ~/.ssh" 
	scp pi@raspberrypi etc/authorized_keys pi@raspberrypi:.ssh/
	#from now on you shouldn't need the password
	scp pi@raspberrypi etc/id_rsa* pi@raspberrypi:.ssh/
	ssh pi@raspberrypi "sudo mkdir -p /root/.ssh && sudo cp ~/.ssh/authorized_keys /root/.ssh"

	# Rename it to tglight01
	#### NO, THIS DOESNT WORK!!!! ssh root@raspberrypi "echo "tglight01" > /etc/host && sed -E 's/raspberrypi/tglight01/' /etc/hosts > /etc/hosts.new && rm /etc/hosts && rm /etc/hosts.new /etc/hosts"

	ssh root@raspberrypi -t "mkdir -p /etc/X11/xorg.conf.d/" 
	scp etc/screencalibration.conf root@raspberrypi:/etc/X11/xorg.conf.d/99-calibration.conf

	# Change pi user password to something else so that it does not complain it's unsecure
	scp etc/shadow root@raspberrypi:/etc/shadow

	# Set up unclutter
	ssh -t pi@raspberrypi "sudo apt-get install -y unclutter"
	ssh -t pi@raspberrypi "mkdir -p ~/.config/lxsession/LXDE-pi/"
	scp etc/autostart pi@raspberrypi:~/.config/lxsession/LXDE-pi/autostart

	# Set up autorun script on a USB stick
	ssh -t pi@raspberrypi "echo \"@reboot /bin/bash /media/pi/AUTORUN/autorun.sh\" | crontab -"

```



## Good to know

Installing touch screen driviers for Elecrow display:

```shell
	git clone https://github.com/Elecrow-keen/Elecrow-LCD5.git
	cd Elecrow-LCD5
	sudo ./Elecrow-LCD5
``` 

OR

```

Then edit /boot/config.txt to add the following lines:
----------------------------------------------------------------------------------
hdmi_group=2
hdmi_mode=1
hdmi_mode=87
hdmi_cvt 800 400 60 6 0 0 0
dtparam=spi=on
dtparam=i2c_arm=on

dtoverlay=ads7846,cs=1,penirq=25,penirq_pull=2,speed=50000,keep_vref_on=0,swapxy=0,pmax=255,xohms=150,xmin=200,xmax=3900,ymin=200,ymax=3900 <======Note this is all on 1 line!

dtoverlay=w1-gpio-pullup,gpiopin=4,extpullup=1

....

sudo apt-get install xinput-calibrator
sudo DISPLAY=:0.0 xinput_calibrator

```