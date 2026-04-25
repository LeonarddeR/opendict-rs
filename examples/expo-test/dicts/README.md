# Test dictionaries

The contents of this directory are git-ignored. Drop dictionaries here as
subdirectories — one subdirectory per dictionary — and they'll be pushed
to the simulator/emulator and exercised by the test app.

## Layout

```
dicts/
  my-stardict-dict/
    foo.ifo
    foo.idx
    foo.dict
    foo.syn        (optional)
  my-mdict-dict/
    bar.mdx
    bar.mdd        (optional)
```

Each subdirectory must contain either a StarDict set (`.ifo` + `.idx` +
`.dict[.gz|.dz]` + optional `.syn`) or an MDict set (`.mdx` + optional
`.mdd`). Format is auto-detected.

## Quick setup

To populate this directory with the bundled fixtures and any local
dictionaries from `tests/dicts/`:

```bash
./scripts/setup-dicts.sh
```

Then push them to the running simulator/emulator:

```bash
./scripts/push-dicts.sh
```

Reload the app — it lists every subdirectory here and runs an assertion
suite against each.
