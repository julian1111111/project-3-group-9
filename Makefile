.PHONY: all clean

all:
	cargo build --release
	cp target/release/filesys filesys

clean:
	cargo clean
	rm -f filesys
