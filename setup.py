"""Build the native ABI into a platform-specific, Python-version-independent wheel."""
from pathlib import Path
import shutil
import subprocess
import sys
from setuptools import Distribution, setup
from setuptools.command.build_py import build_py
from setuptools.command.bdist_wheel import bdist_wheel

ROOT = Path(__file__).parent.resolve()


class BuildPy(build_py):
    def run(self):
        # An explicit target and target-dir override Cargo environment/config redirects.
        host = next(line.split(": ", 1)[1] for line in subprocess.check_output(["rustc", "-vV"], text=True).splitlines() if line.startswith("host: "))
        native_target = ROOT / "target" / "wheel-native"
        subprocess.run(["cargo", "build", "--manifest-path", str(ROOT / "Cargo.toml"), "--release", "--features", "ffi", "--locked", "--target-dir", str(native_target), "--target", host], check=True)
        super().run()
        filename = "libabstractavatar.dylib" if sys.platform == "darwin" else "abstractavatar.dll" if sys.platform == "win32" else "libabstractavatar.so"
        destination = Path(self.build_lib) / "abstractavatar" / "_native"
        destination.mkdir(parents=True, exist_ok=True)
        for old_name in ("libabstractavatar.dylib", "libabstractavatar.so", "abstractavatar.dll"):
            (destination / old_name).unlink(missing_ok=True)
        shutil.copy2(native_target / host / "release" / filename, destination / filename)


class PlatformWheel(bdist_wheel):
    def finalize_options(self):
        super().finalize_options()
        self.root_is_pure = False

    def get_tag(self):
        _, _, platform = super().get_tag()
        return "py3", "none", platform


class BinaryDistribution(Distribution):
    def has_ext_modules(self):
        return True


setup(distclass=BinaryDistribution, cmdclass={"build_py": BuildPy, "bdist_wheel": PlatformWheel})
