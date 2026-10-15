"""Package one prebuilt binary using the filenames expected by both installers."""

import argparse
import hashlib
from pathlib import Path
import tarfile
import zipfile


def package(binary: Path, platform: str, arch: str, output: Path) -> Path:
    output.mkdir(parents=True, exist_ok=True)
    name = f"muhan-{platform}-{arch}"
    if platform == "windows":
        archive = output / f"{name}.zip"
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as bundle:
            bundle.write(binary, "muhan.exe")
    else:
        archive = output / f"{name}.tar.gz"
        with tarfile.open(archive, "w:gz") as bundle:
            info = bundle.gettarinfo(str(binary), arcname="muhan")
            info.mode = 0o755
            with binary.open("rb") as source:
                bundle.addfile(info, source)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_name(archive.name + ".sha256").write_text(
        f"{digest}  {archive.name}\n", encoding="ascii"
    )
    return archive


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--platform", choices=["linux", "macos", "windows"], required=True)
    parser.add_argument("--arch", choices=["x86_64", "aarch64"], required=True)
    parser.add_argument("--output", type=Path, default=Path("dist"))
    args = parser.parse_args()
    print(package(args.binary, args.platform, args.arch, args.output))
