use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

struct TestRepository {
    temp: TempDir,
    root: PathBuf,
    remote: PathBuf,
    program_home: PathBuf,
    global_config: PathBuf,
}

#[test]
fn global_install_does_not_break_later_git_init() {
    let temp = tempfile::tempdir().unwrap();
    let program_home = temp.path().join("program-home");
    let global_config = temp.path().join("global.gitconfig");

    let configured = |program: &str| {
        let mut command = Command::new(program);
        command
            .current_dir(temp.path())
            .env("PUSHVEIL_HOME", &program_home)
            .env("GIT_CONFIG_GLOBAL", &global_config)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("NO_COLOR", "1");
        command
    };

    let mut install = configured(env!("CARGO_BIN_EXE_pushveil"));
    let install = install.arg("install").output().unwrap();
    assert!(
        install.status.success(),
        "{}",
        String::from_utf8_lossy(&install.stderr)
    );

    let bare = temp.path().join("remote.git");
    let mut init_bare = configured("git");
    let init_bare = init_bare
        .args(["init", "--bare", bare.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        init_bare.status.success(),
        "bare git init failed after installation:\n{}",
        String::from_utf8_lossy(&init_bare.stderr)
    );

    let work = temp.path().join("work");
    let mut init_work = configured("git");
    let init_work = init_work
        .args(["init", "--initial-branch=main", work.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        init_work.status.success(),
        "working-tree git init failed after installation:\n{}",
        String::from_utf8_lossy(&init_work.stderr)
    );
}

impl TestRepository {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("work");
        let remote = temp.path().join("remote.git");
        let program_home = temp.path().join("program-home");
        let global_config = temp.path().join("global.gitconfig");
        fs::create_dir_all(&root).unwrap();

        let repository = Self {
            temp,
            root,
            remote,
            program_home,
            global_config,
        };
        repository.git_at(
            repository.temp.path(),
            ["init", "--bare", repository.remote.to_str().unwrap()],
        );
        repository.git(["init", "--initial-branch=main"]);
        repository.git(["config", "user.name", "Secret Guard Test"]);
        repository.git(["config", "user.email", "test@example.invalid"]);
        repository.git([
            "remote",
            "add",
            "origin",
            repository.remote.to_str().unwrap(),
        ]);
        repository.install();
        repository
    }

    fn install(&self) {
        let output = self.program_command().arg("install").output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn write(&self, path: &str, content: &[u8]) {
        let destination = self.root.join(path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(destination, content).unwrap();
    }

    fn remove(&self, path: &str) {
        fs::remove_file(self.root.join(path)).unwrap();
    }

    fn commit_all(&self, message: &str) {
        self.git(["add", "-A"]);
        self.git(["commit", "-m", message]);
    }

    fn git<const N: usize>(&self, args: [&str; N]) -> Output {
        self.git_at(&self.root, args)
    }

    fn git_at<const N: usize>(&self, directory: &Path, args: [&str; N]) -> Output {
        let output = self
            .base_command("git")
            .current_dir(directory)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn push(&self) -> Output {
        self.base_command("git")
            .current_dir(&self.root)
            .args(["push", "origin", "HEAD:refs/heads/main"])
            .output()
            .unwrap()
    }

    fn program_command(&self) -> Command {
        let mut command = self.base_command(env!("CARGO_BIN_EXE_pushveil"));
        command.current_dir(&self.root);
        command
    }

    fn base_command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        command
            .env("PUSHVEIL_HOME", &self.program_home)
            .env("GIT_CONFIG_GLOBAL", &self.global_config)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("NO_COLOR", "1");
        command
    }
}

#[test]
fn safe_push_succeeds_and_existing_repository_hook_still_runs() {
    let repository = TestRepository::new();
    let marker = repository.root.join("hook-ran.txt");
    let hook = repository.root.join(".git/hooks/pre-push");
    fs::write(
        &hook,
        format!(
            "#!/bin/sh\nprintf ran > '{}'\n",
            marker.to_string_lossy().replace('\\', "/")
        ),
    )
    .unwrap();
    make_executable(&hook);

    repository.write("README.md", b"safe content\n");
    repository.commit_all("safe commit");
    let push = repository.push();

    assert!(
        push.status.success(),
        "{}",
        String::from_utf8_lossy(&push.stderr)
    );
    assert!(
        marker.is_file(),
        "the existing repository pre-push hook was not chained"
    );
    assert!(String::from_utf8_lossy(&push.stderr).contains("no secrets found"));
}

#[test]
fn non_interactive_agent_push_blocks_secret_from_earlier_deleted_commit() {
    let repository = TestRepository::new();
    repository.write("README.md", b"safe content\n");
    repository.commit_all("baseline");
    assert!(repository.push().status.success());

    let secret = [
        b"OPENAI_API_KEY=sk-proj-".as_slice(),
        b"abcdefghijklmnopqrstuvwxyz0123456789\n".as_slice(),
    ]
    .concat();
    repository.write("config/private.env", &secret);
    repository.commit_all("accidentally add secret");
    repository.remove("config/private.env");
    repository.commit_all("delete secret file");

    let push = repository.push();
    let stderr = String::from_utf8_lossy(&push.stderr);
    assert!(!push.status.success(), "the push unexpectedly succeeded");
    assert!(stderr.contains("Push blocked"), "{stderr}");
    assert!(stderr.contains("config/private.env:1"), "{stderr}");
    assert!(stderr.contains("No interactive terminal"), "{stderr}");
}

#[test]
fn secret_remains_detectable_after_a_later_rename() {
    let repository = TestRepository::new();
    repository.write("README.md", b"safe content\n");
    repository.commit_all("baseline");
    assert!(repository.push().status.success());

    let secret = [
        b"SERVICE_TOKEN=sk-proj-".as_slice(),
        b"zyxwvutsrqponmlkjihgfedcba9876543210\n".as_slice(),
    ]
    .concat();
    repository.write("old/location.env", &secret);
    repository.commit_all("add configuration");
    fs::create_dir_all(repository.root.join("new")).unwrap();
    repository.git(["mv", "old/location.env", "new/location.env"]);
    repository.git(["commit", "-m", "move configuration"]);

    let push = repository.push();
    let stderr = String::from_utf8_lossy(&push.stderr);
    assert!(!push.status.success());
    assert!(stderr.contains("old/location.env:1"), "{stderr}");
}

#[test]
fn binary_blob_is_scanned_instead_of_skipped() {
    let repository = TestRepository::new();
    let mut binary = vec![0, 1, 2, 3];
    binary.extend_from_slice(b"AKIA");
    binary.extend_from_slice(b"ABCDEFGHIJKLMNOP");
    binary.extend_from_slice(&[0, 255, 10]);
    repository.write("assets/archive.bin", &binary);
    repository.commit_all("binary asset");

    let push = repository.push();
    let stderr = String::from_utf8_lossy(&push.stderr);
    assert!(!push.status.success());
    assert!(stderr.contains("assets/archive.bin (byte 4)"), "{stderr}");
}

#[test]
fn local_git_lfs_content_is_resolved_and_scanned() {
    let repository = TestRepository::new();
    if !repository
        .base_command("git")
        .args(["lfs", "version"])
        .output()
        .is_ok_and(|output| output.status.success())
    {
        eprintln!("git-lfs is unavailable; skipping LFS integration coverage");
        return;
    }

    let oid = "25e0fad9477ce1e1cea97bfb6af0cf6ef42ad40175171dabfd60c603b5543cb4";
    let secret = [
        b"OPENAI_API_KEY=sk-proj-".as_slice(),
        b"abcdefghijklmnopqrstuvwxyz0123456789\n".as_slice(),
    ]
    .concat();
    let object_path = repository.root.join(".git/lfs/objects/25/e0").join(oid);
    fs::create_dir_all(object_path.parent().unwrap()).unwrap();
    fs::write(object_path, &secret).unwrap();
    let pointer = format!(
        "version https://git-lfs.github.com/spec/v1\noid sha256:{oid}\nsize {}\n",
        secret.len()
    );
    repository.write("models/credentials.bin", pointer.as_bytes());
    repository.commit_all("add LFS pointer");

    let push = repository.push();
    let stderr = String::from_utf8_lossy(&push.stderr);
    assert!(!push.status.success());
    assert!(stderr.contains("models/credentials.bin:1"), "{stderr}");
    assert!(stderr.contains("Git LFS object"), "{stderr}");
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(not(unix))]
const fn make_executable(_path: &Path) {}
