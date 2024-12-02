.PHONY: all clean

all:
	cargo build --release
	./target/release/filesys fat32.img

clean:
	cargo clean
	rm -f filesys
