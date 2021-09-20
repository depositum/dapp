#!/usr/bin/env bash

set -o errexit

ROOT_PATH=$(cd "$( dirname "${BASH_SOURCE[0]}" )" && cd ../../ && pwd)

# cargo install wasm-snip wasm-gc
# apt-get install -y binaryen wabt
# https://github.com/near/near-sdk-rs/tree/master/minifier
minify() {
  if [ "$(command -v wasm-snip wasm-gc wasm-strip wasm-opt | wc -l)" != 4 ]; then
    echo "Minifying skipped"
    return
  fi
  filePath="${1}"
  fileName=$(basename -- "${filePath}")
  dirPath=$(dirname -- "${filePath}")
  tmpPath="${dirPath}/temp-${fileName}"
  outFileName="${fileName%.*}.${fileName##*.}"
  outPath="${dirPath}/${outFileName}"
  wasm-snip \
    --snip-rust-fmt-code \
    --snip-rust-panicking-code \
    --pattern core::num::flt2dec::.* \
    --pattern core::fmt::float::.* \
    --output "${tmpPath}" \
    "${filePath}"
  wasm-gc "${tmpPath}"
  wasm-strip "${tmpPath}"
  wasm-opt -Oz "${tmpPath}" --output "${outPath}"
  rm "${tmpPath}"
  fileSize=$(stat -c "%s" "${filePath}")
  outSize=$(stat -c "%s" "${outPath}")
  echo "Minifying ${fileName} ${fileSize} bytes -> ${outSize} bytes, see ${outFileName}"
}

build() {
  package="${1}"
  cargo build --package "${package}" --target wasm32-unknown-unknown --release
  mkdir -p build
  path="build/${package}.wasm"
  cp target/wasm32-unknown-unknown/release/*.wasm build/
  echo "${path}"
  minify "${path}"
  printf "size: %s\n" "$(du -b "${path}" | cut -f1)"
  printf "hash: %s\n" "$(sha256sum "${path}" | cut -d' ' -f1)"
}

cd "$ROOT_PATH"
build depositum
build simple_token
build ref_farming_strategy

# FIXME setup not root user in docker
HOST_OWNER=${HOST_OWNER:-"$(id -u):$(id -g)"}
chown "$HOST_OWNER" -R "$ROOT_PATH"
