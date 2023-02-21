run:
	mv Clans.txt Clans.txt.old
	cargo run --release

windows_release:
	export RUSTFLAGS="-C opt-level=3"
	cargo build --target x86_64-pc-windows-gnu --release
