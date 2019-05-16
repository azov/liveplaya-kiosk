run raspi config

run script

fix boot/cmdline.txt by putting everything on one line and removing duplicate entries

reboot

fix iptables by running sudo su, copy/pasting lines 240-249

copy over all files to /opt/gecko/bms

create symlink to 
  ln -s /opt/gecko/bms /opt/gecko/live

create symlink for HTML page
  cd /opt/gecko/bms
  ln -s ./Nuvation-BMS-(new CSK OI name)....html index.html

run read-only script

run fix-rc-local.sh

sudo raspi-config make boot options autologin

reboot

attach jumper across pins 21 and GND (straight across) and reboot

on the pi, login, let the UI load, press CTRL- to scale page to 80%, use CTRL-w to close page

reboot