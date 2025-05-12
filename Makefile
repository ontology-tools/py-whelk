build:
	cargo build --release
	cp target/release/*.so target/release/*.dll pywhelk/ || true
	python3 -m build
