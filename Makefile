ifeq (test, $(firstword $(MAKECMDGOALS)))
  runargs := $(wordlist 2, $(words $(MAKECMDGOALS)), $(MAKECMDGOALS))
  $(eval $(runargs):;@true)
endif

.PHONY: spectests emtests clean build install lint precommit

# This will re-generate the Rust test files based on spectests/*.wast
spectests:
	WASMER_RUNTIME_GENERATE_SPECTESTS=1 cargo build -p wasmer-runtime-core

emtests:
	WASM_EMSCRIPTEN_GENERATE_EMTESTS=1 cargo build -p wasmer-emscripten

# clean:
#     rm -rf artifacts

build:
	cargo build --features debug

install:
	cargo install --path .

integration-tests: release
	echo "Running Integration Tests"
	./integration_tests/lua/test.sh
	./integration_tests/nginx/test.sh

lint:
	cargo fmt --all -- --check
	cargo clippy --all

precommit: lint test

test:
	# We use one thread so the emscripten stdouts doesn't collide
	cargo test --all --exclude wasmer-runtime-c-api --exclude wasmer-emscripten --exclude wasmer-spectests -- $(runargs)
	# cargo test --all --exclude wasmer-emscripten -- --test-threads=1 $(runargs)
	cargo test --manifest-path lib/spectests/Cargo.toml --features clif
	cargo test --manifest-path lib/spectests/Cargo.toml --features llvm
	cargo build -p wasmer-runtime-c-api
	cargo test -p wasmer-runtime-c-api -- --nocapture

test-emscripten:
	cargo test --manifest-path lib/emscripten/Cargo.toml --features clif -- --test-threads=1 $(runargs)
	cargo test --manifest-path lib/emscripten/Cargo.toml --features llvm -- --test-threads=1 $(runargs)

release:
	# If you are in OS-X, you will need mingw-w64 for cross compiling to windows
	# brew install mingw-w64
	cargo build --release

debug-release:
	cargo build --release --features debug

debug-release:
	cargo build --release --features "debug"

publish-release:
	ghr -t ${GITHUB_TOKEN} -u ${CIRCLE_PROJECT_USERNAME} -r ${CIRCLE_PROJECT_REPONAME} -c ${CIRCLE_SHA1} -delete ${VERSION} ./artifacts/

visualization:
	gource --stop-at-end --default-user-image images/default.png --user-image-dir images  --user-scale 1.5 --bloom-intensity 0.3 --logo logo-small.png --hide filenames --dir-name-depth 2 --background-image background2.jpg --font-colour 000000 --dir-colour 333333 --selection-colour 533AC9 --highlight-colour 533AC9 --seconds-per-day 0.1 --auto-skip-seconds 0.3 -1280x720 --file-filter spectests --file-filter emtests -o - | ffmpeg -y -r 60 -f image2pipe -vcodec ppm -i - -vcodec libx264 -preset ultrafast -pix_fmt yuv420p -crf 1 -threads 0 -bf 0 gource.mp4
