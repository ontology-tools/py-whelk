build:
	cargo build --release
	cp target/release/*.so target/release/*.dll pywhelk/ || true
	python3 -m build

build-dev:
	cargo build
	cp target/debug/*.so target/debug/*.dll pywhelk/ || true
	python3 -m build