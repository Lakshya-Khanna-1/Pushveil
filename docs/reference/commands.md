# Command reference

```text
pushveil <COMMAND>
```

Use `pushveil --help` or `pushveil <command> --help` for the version-specific CLI output.

## `pushveil install`

Installs the running executable for the current user, writes global hook wrappers, stores the previous hook path, and activates Pushveil through Git's global `core.hooksPath`.

```bash
pushveil install
```

The operation is idempotent for the same installation. Running a newer version refreshes the versioned executable and wrappers while retaining the original pre-Pushveil hook path.

Exit status is `0` on success and nonzero when Git is unavailable or installation/configuration writes fail.

## `pushveil doctor`

Checks:

- Git can run;
- installation state can be loaded;
- the versioned installed binary exists;
- the `pre-push` wrapper exists;
- global `core.hooksPath` points to the recorded Pushveil directory.

```bash
pushveil doctor
```

Exit status is `0` when healthy and `1` when any health check fails.

## `pushveil scan [REVISION]`

Scans commits returned by `git rev-list` for the supplied revision expression. The default is `HEAD`.

```bash
pushveil scan
pushveil scan HEAD
pushveil scan origin/main..HEAD
pushveil scan v1.0.0..v1.1.0
pushveil scan -- --all
```

The `--` separator is required when the revision expression itself begins with a hyphen.

Exit status is `1` when findings exist or fail-closed verification errors occur; otherwise it is `0`.

### `--json`

```bash
pushveil scan origin/main..HEAD --json
```

Outputs a JSON object containing:

- `findings`: rule, description, path, commit, object ID, line or byte offset, binary flag, and LFS flag;
- `errors`: verification errors that did not prevent result construction;
- `stats`: commits, blobs, LFS objects, submodule references, and bytes scanned.

Treat the schema as versioned product output and pin the Pushveil version in automation that parses it.

## `pushveil uninstall`

Restores the recorded global hook path, or removes Pushveil's global setting when no global path previously existed.

```bash
pushveil uninstall
```

The command refuses to overwrite `core.hooksPath` if it no longer points to the recorded Pushveil directory. Cached binaries remain inert after routing is restored.

## Internal `hook` command

The generated wrappers call a hidden `pushveil hook <hook-name> ...` command. It is an implementation detail, not a stable public interface. Scripts should use the documented commands above.

