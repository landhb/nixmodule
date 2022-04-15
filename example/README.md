## NixModule Example Module

The `nixmodule-config.toml` is normally located in the directory with the `Makefile`, unless using the `--config` option to specify an alternative. `nixmodule` must be ran in the same directory as the `Makefile`.

As an example, when building for linux-5.17.2, the `Makefile` will be passed `TARGET=example-5.17.2` and `KERNEL=~/.cache/nixmodule/cache/5.17.2/headers`.

### Using nixmodule


```sh
cargo install nixmodule
cd example
nixmodule
```