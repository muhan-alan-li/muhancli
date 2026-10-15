"""Exercise the real installer and binary with local, simulated release downloads."""

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("MUHAN_TEST_BINARY", ROOT / "target/release/muhan")).resolve()


class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="muhan-installer-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.home = self.root / "home with ' quotes"
        self.home.mkdir()
        self.tools = self.root / "tools"
        self.tools.mkdir()
        # Explicit tool directory lets us exercise wget without finding system curl.
        for tool in ["awk", "chmod", "cp", "grep", "gzip", "head", "mkdir", "mktemp", "mv", "rm", "sed", "tar", "tr", "shasum", "sha256sum", "perl"]:
            executable = shutil.which(tool)
            if executable:
                (self.tools / tool).symlink_to(executable)
        self.env = dict(os.environ, HOME=str(self.home), SHELL="/bin/bash",
                        PATH=str(self.tools), FIXTURES=str(self.root),
                        MOCK_OS="Linux", MOCK_ARCH="x86_64")
        for key in ["DEST", "VERSION", "REPO", "NO_MODIFY_PATH", "ZDOTDIR"]:
            self.env.pop(key, None)
        self.write_tool("uname", '#!/bin/sh\ncase "$1" in -s) echo "$MOCK_OS";; -m) echo "$MOCK_ARCH";; esac\n')
        self.mock_download = f"#!{sys.executable}\n" + '''
import json, os, pathlib, shutil, sys
args = sys.argv[1:]
option = '-o' if '-o' in args else '-qO'
destination = pathlib.Path(args[args.index(option) + 1])
url = args[-1]
root = pathlib.Path(os.environ['FIXTURES'])
with (root / 'requests').open('a') as log:
    log.write(url + '\\n')
if url.endswith('/releases/latest'):
    destination.write_text(json.dumps({'tag_name': 'v0.1.0'}))
else:
    source = root / ('expected.sha256' if url.endswith('.sha256') else 'release.tar.gz')
    if not source.exists():
        sys.exit(22)
    shutil.copyfile(source, destination)
'''
        self.write_tool("curl", self.mock_download)
        with tarfile.open(self.root / "release.tar.gz", "w:gz") as archive:
            archive.add(BINARY, arcname="muhan")
        self.set_checksum()

    def write_tool(self, name, contents):
        path = self.tools / name
        path.write_text(contents)
        path.chmod(0o755)

    def set_checksum(self):
        digest = hashlib.sha256((self.root / "release.tar.gz").read_bytes()).hexdigest()
        (self.root / "expected.sha256").write_text(f"{digest}  muhan.tar.gz\n")

    def install(self, shell="/bin/sh", success=True):
        result = subprocess.run([shell, str(ROOT / "install.sh")], env=self.env,
                                capture_output=True, text=True)
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    @property
    def installed(self):
        return Path(self.env.get("DEST", self.home / ".local/bin")) / "muhan"

    def test_install_and_run_with_quoted_path(self):
        self.install()
        output = subprocess.check_output([self.installed, "--json", "about"], text=True)
        self.assertTrue(json.loads(output)["name"])
        profile = self.home / ".profile"
        result = subprocess.check_output(
            ["/bin/sh", "-c", '. "$1"; command -v muhan', "sh", str(profile)],
            env=self.env, text=True)
        self.assertEqual(result.strip(), str(self.installed))
        requests = (self.root / "requests").read_text()
        self.assertIn("/download/v0.1.0/muhan-linux-x86_64.tar.gz", requests)
        self.assertNotIn("/latest/download/", requests)

    def test_reinstall_is_idempotent(self):
        self.install("/bin/bash")
        self.install("/bin/bash")
        self.assertEqual((self.home / ".bashrc").read_text().count("export PATH="), 1)
        self.assertEqual((self.home / ".profile").read_text().count("export PATH="), 1)

    def test_zsh_and_custom_destination(self):
        self.env.update(SHELL="/bin/zsh", DEST=str(self.home / "custom bin"))
        self.install()
        self.assertTrue(self.installed.is_file())
        resolved = subprocess.check_output(
            ["/bin/sh", "-c", '. "$1"; command -v muhan', "sh", str(self.home / ".zshrc")],
            env=self.env, text=True)
        self.assertEqual(resolved.strip(), str(self.installed))

    def test_wget_and_pinned_version(self):
        (self.tools / "curl").unlink()
        self.write_tool("wget", self.mock_download)
        self.env["VERSION"] = "v0.1.0"
        self.install()
        self.assertNotIn("/releases/latest", (self.root / "requests").read_text())

    def test_macos_arm_asset_selection(self):
        self.env.update(MOCK_OS="Darwin", MOCK_ARCH="arm64")
        self.install()
        self.assertIn("muhan-macos-aarch64.tar.gz", (self.root / "requests").read_text())

    def test_no_modify_path(self):
        self.env["NO_MODIFY_PATH"] = "1"
        self.install()
        self.assertFalse((self.home / ".bashrc").exists())
        self.assertFalse((self.home / ".profile").exists())

    def test_bad_checksum_preserves_existing_install(self):
        self.install()
        before = self.installed.read_bytes()
        (self.root / "expected.sha256").write_text("0" * 64 + "  muhan.tar.gz\n")
        result = self.install(success=False)
        self.assertIn("Checksum mismatch", result.stderr)
        self.assertEqual(self.installed.read_bytes(), before)

    def test_missing_checksum_fails_closed(self):
        (self.root / "expected.sha256").unlink()
        self.install(success=False)
        self.assertFalse(self.installed.exists())

    def test_invalid_platform_and_version(self):
        self.env["MOCK_ARCH"] = "i686"
        self.install(success=False)
        self.env.update(MOCK_ARCH="x86_64", VERSION="v0.1.0/../../bad")
        self.install(success=False)
        self.assertFalse(self.installed.exists())

    def test_missing_binary_in_archive(self):
        with tarfile.open(self.root / "release.tar.gz", "w:gz"):
            pass
        self.set_checksum()
        self.install(success=False)
        self.assertFalse(self.installed.exists())


if __name__ == "__main__":
    unittest.main()
