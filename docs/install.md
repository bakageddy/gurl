# Installation

## Packages

Download the package for your platform from the Releases page of the repository.

| Platform | Package |
| --- | --- |
| macOS, Apple Silicon and Intel | `.dmg` |
| Windows | NSIS installer `.exe` |
| Ubuntu 22.04 and 24.04, Debian | `.deb` or `.AppImage` |
| Fedora | `.rpm` |
| Arch Linux | tarball, or `packaging/arch/PKGBUILD` |

The HURL engine is part of the application. You do not install HURL separately.

## Runtime dependencies

gURL uses three native libraries from the operating system: libcurl, libxml2 and OpenSSL. The `.deb`, the
`.rpm` and the Arch `PKGBUILD` declare them. The package manager installs them. For the AppImage and the Arch
tarball, install them yourself.

| Platform | Packages |
| --- | --- |
| Ubuntu, Debian | `libcurl4 libxml2 libssl3` (`libssl3t64` on Ubuntu 24.04) |
| Fedora | `libcurl libxml2 openssl-libs` |
| Arch Linux | `curl libxml2 openssl` |
| macOS | None. libcurl and libxml2 are part of the OS. |
| Windows | None. The installer contains all libraries. |

To see which libraries are in use, run:

```sh
gurl --version
```

The output shows the gURL version, the HURL version, the libcurl version line, and for libcurl and libxml2
whether the library is linked statically or dynamically. For a dynamic library, the output shows its path.

## Command line access

The application binary is also the command line tool. See [Command line](cli.md).

- Linux packages install `/usr/bin/gurl`.
- On macOS, create a link:

  ```sh
  ln -s /Applications/gURL.app/Contents/MacOS/gurl /usr/local/bin/gurl
  ```

- On Windows, add the installation directory to `PATH`. The application is a GUI program. When you run a
  command without a pipe or a redirection, the prompt can return before the output. Use a pipe, for example
  `gurl cli --test a.hurl | more`, to make the terminal wait.
