#!/bin/bash -e
# ==============================================================================
# Hardware group permissions for the dashboard and Bluetooth stack
# ==============================================================================

# Keep the image configurable: do not hardcode the default pi account.
for grp in video dialout bluetooth input tty audio render; do
    if getent group "$grp" > /dev/null 2>&1; then
        usermod -a -G "$grp" "${FIRST_USER_NAME}"
    fi
done
