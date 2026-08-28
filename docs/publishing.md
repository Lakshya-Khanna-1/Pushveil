# Publishing checklist

The repository is ready to publish. Before cutting a release:

1. Enable private vulnerability reporting and branch protection requiring the CI jobs on `main`.

2. Confirm all workflows pass on Linux, macOS, and Windows.

3. Create the first release by tagging the exact version from `Cargo.toml`:

   ```bash
   git tag -s v0.1.0 -m "Pushveil 0.1.0"
   git push origin v0.1.0
   ```

4. Download each generated archive and checksum from the release, verify the checksum, and perform an install/push/uninstall smoke test on each supported operating system.

5. For enterprise distribution, sign the Windows executable, notarize the macOS binaries, and publish the signing identities and verification procedure.

Do not publish a release from a dirty working tree or when the version and changelog disagree.
