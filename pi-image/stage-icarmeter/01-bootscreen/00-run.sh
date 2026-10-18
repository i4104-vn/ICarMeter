#!/bin/bash -e
# ==============================================================================
# Host-side: Copy Plymouth theme files into rootfs
# ==============================================================================

THEME_DIR="${ROOTFS_DIR}/usr/share/plymouth/themes/icarmeter"
install -d -m 755 "${THEME_DIR}"
cp -r files/icarmeter/* "${THEME_DIR}/"
chmod 644 "${THEME_DIR}"/*
