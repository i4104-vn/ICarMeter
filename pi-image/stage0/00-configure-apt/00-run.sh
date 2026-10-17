#!/bin/bash -e

true > "${ROOTFS_DIR}/etc/apt/sources.list"
install -m 644 files/debian.sources "${ROOTFS_DIR}/etc/apt/sources.list.d/"
install -m 644 files/raspi.sources "${ROOTFS_DIR}/etc/apt/sources.list.d/"
sed -i "s/RELEASE/${RELEASE}/g" "${ROOTFS_DIR}/etc/apt/sources.list.d/debian.sources"
sed -i "s/RELEASE/${RELEASE}/g" "${ROOTFS_DIR}/etc/apt/sources.list.d/raspi.sources"

if [ -n "$APT_PROXY" ]; then
	install -m 644 files/51cache "${ROOTFS_DIR}/etc/apt/apt.conf.d/51cache"
	sed "${ROOTFS_DIR}/etc/apt/apt.conf.d/51cache" -i -e "s|APT_PROXY|${APT_PROXY}|"
else
	rm -f "${ROOTFS_DIR}/etc/apt/apt.conf.d/51cache"
fi

if [ -n "$TEMP_REPO" ]; then
	install -m 644 /dev/null "${ROOTFS_DIR}/etc/apt/sources.list.d/00-temp.list"
	echo "$TEMP_REPO" | sed "s/RELEASE/$RELEASE/g" > "${ROOTFS_DIR}/etc/apt/sources.list.d/00-temp.list"
else
	rm -f "${ROOTFS_DIR}/etc/apt/sources.list.d/00-temp.list"
fi

install -m 644 files/raspberrypi-archive-keyring.pgp "${ROOTFS_DIR}/usr/share/keyrings/"

# Ensure Debian archive keyrings are copied from host into rootfs
if [ -d /usr/share/keyrings ]; then
	cp -a /usr/share/keyrings/debian-archive* "${ROOTFS_DIR}/usr/share/keyrings/" 2>/dev/null || true
fi

# Ensure both .gpg and .pgp exist so apt can verify regardless of extension
if [ -f "${ROOTFS_DIR}/usr/share/keyrings/debian-archive-keyring.gpg" ] && [ ! -f "${ROOTFS_DIR}/usr/share/keyrings/debian-archive-keyring.pgp" ]; then
	ln -sf debian-archive-keyring.gpg "${ROOTFS_DIR}/usr/share/keyrings/debian-archive-keyring.pgp"
fi
if [ -f "${ROOTFS_DIR}/usr/share/keyrings/debian-archive-keyring.pgp" ] && [ ! -f "${ROOTFS_DIR}/usr/share/keyrings/debian-archive-keyring.gpg" ]; then
	ln -sf debian-archive-keyring.pgp "${ROOTFS_DIR}/usr/share/keyrings/debian-archive-keyring.gpg"
fi
on_chroot <<- \EOF
	ARCH="$(dpkg --print-architecture)"
	if [ "$ARCH" = "armhf" ]; then
		dpkg --add-architecture arm64
	elif [ "$ARCH" = "arm64" ]; then
		dpkg --add-architecture armhf
	fi
	apt-get update
	apt-get dist-upgrade -y
EOF
