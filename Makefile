.PHONY: test build build-dev

build:
	cargo build --release
	cp target/release/*.so  pywhelk/ || true
	cp target/release/*.dll pywhelk/ || true
	python3 -m build

build-dev:
	cargo build
	cp target/debug/*.so  pywhelk/ || true
	cp target/debug/*.dll pywhelk/ || true
	python3 -m build

test: build
	[ -d test/venv ] || python3 -m venv test/venv
	. test/venv/bin/activate && pip install --force-reinstall .
	python3 -m unittest discover -s test -p "test_*.py"
	
clean:
	rm -rf test/venv
	rm -rf dist/
	rm -rf *.egg-info
	rm -rf target/
	