# Start the dashboard from the tty1 autologin session.
# getty will create a fresh autologin session if the app exits or crashes.
if [ "$(tty 2>/dev/null)" = "/dev/tty1" ] && [ -x /opt/icarmeter/icarmeter ]; then
    cd /opt/icarmeter
    export UV_USE_IO_URING=0
    exec /opt/icarmeter/icarmeter
fi
