A solver for [Ricochet Robots](https://boardgamegeek.com/boardgame/51/ricochet-robots)

https://github.com/kaseken/ricochet_robots input format is supported.


# Profiling on macOS

install `cargo-instruments` and run:

```
cargo instruments --release --bin profile --template time --time-limit 10000
```

add the following to `Cargo.toml` to get more insights.

```
[profile.release]
debug = true
```
