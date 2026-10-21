#!/bin/bash -e

# Keep Bluetooth available on images that default radios to rfkill-blocked.

mkdir -p "${ROOTFS_DIR}/var/lib/systemd/rfkill/"
#           5                 miniuart 4      miniuart Zero   miniuart other  other
for addr in 107d50c000.serial 3f215040.serial 20215040.serial fe215040.serial soc; do
	echo 0 > "${ROOTFS_DIR}/var/lib/systemd/rfkill/platform-${addr}:bluetooth"
done
