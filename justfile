release:
  cargo zigbuild

release-fast:
  cargo zigbuild --release

release-fast-target target:
  cargo zigbuild --release --target {{target}}

changelog:
  git cliff -o CHANGELOG.md
