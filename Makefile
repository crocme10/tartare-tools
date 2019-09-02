
PROJ_VERSION = 6.1.0

install:
	sudo apt update && sudo apt install -y wget build-essential pkg-config sqlite3 libsqlite3-dev libssl-dev clang
	wget https://github.com/OSGeo/proj.4/releases/download/$(PROJ_VERSION)/proj-$(PROJ_VERSION).tar.gz && tar -xzvf proj-$(PROJ_VERSION).tar.gz
	cd proj-$(PROJ_VERSION) && ./configure --prefix=/usr && sudo make && sudo make install 
	rm -rf proj-$(PROJ_VERSION) proj-$(PROJ_VERSION).tar.gz
