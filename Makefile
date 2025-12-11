APPLET := dev_heppen_tiling_exception_custom
PREFIX := /usr/local

build:
	@echo "Building $(APPLET)..."
	cargo build --release

run: 
	@echo "Running $(APPLET)..."
	./target/release/$(APPLET)

all: build run

install:
	@echo "Installing $(APPLET)..."
	install -Dm0755 ./target/release/$(APPLET) $(PREFIX)/bin/$(APPLET)
	install -Dm0644 ./res/$(APPLET).desktop $(PREFIX)/share/applications/$(APPLET).desktop

uninstall:
	@echo "Uninstalling $(APPLET)..."
	rm $(PREFIX)/bin/$(APPLET)
	rm $(PREFIX)/share/applications/$(APPLET).desktop	
