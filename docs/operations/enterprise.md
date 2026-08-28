# Enterprise deployment

Pushveil provides fast workstation feedback. Enterprise deployment should combine that convenience with an authoritative server-side control.

## Recommended control layers

1. **Developer workstation:** Pushveil blocks mistakes before network transfer.
2. **Pull-request and CI scanning:** independent scanning catches unprotected workstations and imported branches.
3. **Git server or hosting provider:** push protection or a pre-receive hook enforces policy regardless of client behavior.
4. **Credential provider:** short lifetimes, narrow scopes, rotation, and audit logs limit impact.

## Package and sign

Build release binaries from a protected CI pipeline using the locked dependency graph. Publish SHA-256 checksums. Sign Windows binaries, sign and notarize macOS binaries, and use trusted package repositories for Linux.

Record:

- source commit and version;
- build runner image and Rust version;
- dependency-audit result;
- signing identity;
- artifact hashes;
- rollout and rollback owners.

## User-context installation

Deploy the binary through Intune, Jamf, Group Policy, MDM, or configuration management, then run:

```bash
pushveil install
pushveil doctor
```

Run installation as the developer account because `core.hooksPath` is a user-level Git setting. A system-level binary alone does not configure every user's global Git file.

## Environment inventory

Include every place capable of pushing:

- Windows, macOS, and Linux workstations;
- WSL distributions;
- development containers;
- shared build machines;
- VDI and virtual machines;
- SSH workspaces;
- local and cloud coding agents.

Use `pushveil doctor` as a health signal, while recognizing that a user who controls the workstation can still bypass a local hook.

## Organization rules

Maintain high-confidence custom patterns for internal credentials. Review them like authentication code and test them against representative repositories before rollout.

Repository configuration is developer-writable and should not be the sole source of mandatory policy. Enforce required rules on the server, or distribute protected user configuration with filesystem policy while retaining server enforcement.

## Override policy

Pushveil's interactive `PUSH ANYWAY` supports personal and open-source workflows. Organizations that prohibit overrides must enforce the prohibition outside the client—for example through server-side push protection—and instruct agents and developers never to use `--no-verify`.

An optional reason log is not implemented, and Pushveil has no telemetry. Do not claim centralized override auditing without adding and reviewing such a feature.

## Rollout plan

1. Audit common repository types and existing hooks.
2. Pilot with security and volunteer engineering teams.
3. Measure false positives and scan duration without weakening fail-closed behavior.
4. Tune internal rules and documented remediation.
5. Roll out by environment, beginning with users who push sensitive code.
6. Enable the server-side boundary before declaring enforcement complete.
7. Rehearse uninstall and restoration of prior hooks.

## Incident response

A blocked push is a near miss. A credential that reached a remote is an incident. Rotate first, preserve appropriate evidence, use provider audit logs, follow the organization's response process, and coordinate any history rewrite.

