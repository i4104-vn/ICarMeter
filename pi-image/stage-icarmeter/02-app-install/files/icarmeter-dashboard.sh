# Start the dashboard from the tty1 autologin session.
# getty will create a fresh autologin session if the app exits or crashes.
if [ "$(tty 2>/dev/null)" = "/dev/tty1" ] && [ -x /opt/icarmeter/car_dashboard ]; then
    exec /opt/icarmeter/car_dashboard
fi
