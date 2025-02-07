```
rm -r pkg/
RUSTFLAGS="--cfg web_sys_unstable_apis " wasm-pack build --target web --dev
```

```
RUSTFLAGS="--cfg web_sys_unstable_apis " cargo clippy -- -W clippy::pedantic -A clippy::missing_panics_doc -A clippy::cast_possible_truncation -A clippy::missing_errors_doc
```