# This should be in ~/.config/lxsession/LXDE-pi/autostart
if [ -f /media/pi/AUTOSTART/autostart.sh ]
then
	/media/pi/AUTOSTART/autostart.sh
else	
	@lxpanel --profile LXDE-pi
	@pcmanfm --desktop --profile LXDE-pi
	@xscreensaver -no-splash
	@point-rpi
fi
