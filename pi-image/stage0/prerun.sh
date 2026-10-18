#!/bin/bash -e

if [ "$RELEASE" != "bookworm" ]; then
	echo "WARNING: RELEASE is set to '$RELEASE' instead of 'bookworm'."
fi

if [ ! -d "${ROOTFS_DIR}" ]; then
	bootstrap ${RELEASE} "${ROOTFS_DIR}" http://deb.debian.org/debian/
fi
