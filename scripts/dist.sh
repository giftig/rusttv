#!/bin/bash

cd "$(dirname "$0")/.."

# Create a binary distribution for the target platform

PLATFORM="$1"

_archive_name() {
  if [[ "$PLATFORM" == '' ]]; then
    echo "rusttv-$1.$2"
  else
    echo "rusttv-$PLATFORM-$1.$2"
  fi
}

_package_zip() {
  local target_path="$1"
  local version="$2"
  local filename=$(_archive_name "$version" zip)
  mkdir -p dist

  echo "Creating ZIP archive: dist/$filename"
  pushd "$target_path/" >&2
    zip -r "$filename" "$version" || return $?
  popd >&2

  mv "$target_path/$filename" dist/
}

_package_targz() {
  local target_path="$1"
  local version="$2"
  local filename=$(_archive_name "$version" tar.gz)
  mkdir -p dist

  echo "Creating tarball: dist/$filename"
  tar zcf "dist/$filename" $target_path/$version
}

dist_windows() {
  local target_path="target/$PLATFORM"

  if [[ -d "$target_path/debug" ]]; then
    _package_zip "$target_path" debug || return $?
  fi
  if [[ -d "$target_path/release" ]]; then
    _package_zip "$target_path" release || return $?
  fi
}

dist_unix() {
  local target_path='target'
  if [[ "$PLATFORM" != '' ]]; then
    target_path="target/$PLATFORM"
  fi

  if [[ -d "$target_path/debug" ]]; then
    _package_targz "$target_path" debug || return $?
  fi
  if [[ -d "$target_path/release" ]]; then
    _package_targz "$target_path" release || return $?
  fi
}

case "$PLATFORM" in
  *windows*)
    dist_windows
    ;;
  *unix*|"")
    dist_unix
    ;;
  *)
    echo "Unknown platform $PLATFORM" >&2
    exit 1
    ;;
esac


