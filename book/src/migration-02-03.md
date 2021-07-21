# Migrating from `v0.2.x` to `v0.3.0`

This guide covers how to upgrade a library or application using `defmt v0.2.x` to version `v0.3.0`.

## `Cargo.toml`

Update the version of `defmt` to `"0.3"` (or `"0.3.0"`, which is equivalent).

Additionally please remove the `defmt-*` cargo features from your `[features]` section.

```diff
[dependencies]

- defmt = "0.2"
+ defmt = "0.3"

[features]
other-feature = []

- defmt-default = []
- defmt-trace = []
- defmt-debug = []
- defmt-info = []
- defmt-warn = []
- defmt-error = []
```

## Set the log-level with `DEFMT_LOG`

Setting the log-level via cargo features is superseded by the new `DEFMT_LOG` environment variable.

To log everything on the `INFO` level and above, run your application like following:

```console
$ DEFMT_LOG=info cargo run
```

For more details how to configure the log-level using `DEFMT_LOG` see the [user docs](TODO: add link).

## Rename display hint `µs` to `us`

Due to ambiguity in-between `µ` (micro sign) and `μ` (small mu), the display hint for microseconds changed to be `us`.

Therefore you likely need to update your timestamp definition.

```diff
- defmt::timestamp!("{=u32:µs}", {
+ defmt::timestamp!("{=u32:us}", {
    // ...
});
```

As well as all other logging calls where you were using `µs`.

```diff
- defmt::info!("{=u8:µs}", time)
+ defmt::info!("{=u8:us}", time)
```

> 💡 Easily fix this, using the global search-and-replace feature of your editor/IDE. *([vs code](https://code.visualstudio.com/docs/editor/codebasics#_search-and-replace))*

## Drop `u24` type hint

The `u24` type hint got dropped, cause it was confusing users and complicates the code.

Therefore replace it with `u32` in all logging calls.

```diff
- defmt::info!("{=u24}", 42);
+ defmt::info!("{=u32}", 42);
```

> 💡 Use the global search-and-replace here as well!

---

TODO

- [ ] `#505`: Logger trait v2
- [x] `#521`: [3/n] Remove u24
- [x] `#522`: Replace `µs` hint with `us`
- [ ] `#508`: [5/n] Format trait v2
  - no Write trait anymore
- [x] `#519`: `DEFMT_LOG`

