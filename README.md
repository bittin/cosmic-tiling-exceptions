# COSMIC ext Custom Tiling Exceptions

Currently COSMIC desktop does not have direct way to set windows as floating permanently. We need to manually add APPs IDs or APP titles to custom COSMIC config.

This applet trying to simplify this process. If you have window open, this applet after refreshing will look into your opened windows, list them, and allow you by simply button press to add new exception to compositor's config `tiling_exception_custom`.

# Building

`make build`

# Running 

`make run`

or `make all` -> to build and run in one command.

# Installing

`sudo make install`

# Uninstalling

`sudo make uninstall`

