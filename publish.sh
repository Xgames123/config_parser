VERSION=$(rg '^version ?= ?\"([0-9]\.[0-9]\.[0-9])\"$' -r '$1' Cargo.toml)
DEP_VERSION=$(rg '^starryconfig_derive ?= ?\{.*version ?= ?\"([0-9]\.[0-9]\.[0-9])\".*\}$' -r '$1' starryconfig/Cargo.toml)

echo "version: $VERSION"
echo "dep version: $DEP_VERSION"

if [[ "$VERSION" != "$DEP_VERSION" ]] ; then
  echo "Version of dependency doesn't match"
  exit 1
fi

ARGS=""
if [[ "$1" == "--check" ]] ; then
  ARGS="--dry-run"
fi
cargo publish -p starryconfig_derive -p starryconfig $ARGS
