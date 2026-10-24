//! `gurl --version`: gURL, HURL and the native libraries HURL runs on, and how they are linked.

use hurl::http::libcurl_version_info;

/// HURL's version, read from the vendored crate's manifest at build time (see `build.rs`).
pub const HURL_VERSION: &str = env!("GURL_HURL_VERSION");

/// The first line only: `gURL 0.4.0 (x86_64-unknown-linux-gnu)`.
pub fn short() -> String {
    format!(
        "gURL {} ({})",
        env!("CARGO_PKG_VERSION"),
        env!("GURL_TARGET")
    )
}

/// The full report. The second and third lines are exactly what `hurl --version` prints.
pub fn long() -> String {
    let curl = libcurl_version_info();
    let mut out = format!(
        "{}\nhurl {} ({}) {}\nFeatures (libcurl):  {}\nFeatures (built-in): brotli\n",
        short(),
        HURL_VERSION,
        curl.host,
        curl.libraries.join(" "),
        curl.features.join(" ")
    );
    out.push_str(&format!(
        "libcurl:  {}\n",
        linking("curl", curl_sys::vendored())
    ));
    out.push_str(&format!(
        "libxml2:  {}, {}\n",
        libxml2_version(),
        linking("xml2", false)
    ));
    out
}

/// "statically linked" / "dynamically linked (<path>)" for a native library, from the strongest
/// evidence available: a shared object of that name loaded in this process beats the build-time
/// knowledge of whether the crate bundled its own copy.
fn linking(lib: &str, bundled: bool) -> String {
    match loaded_shared_object(lib) {
        Some(path) => format!("dynamically linked ({path})"),
        None if bundled => "statically linked (bundled by curl-sys)".to_string(),
        None if cfg!(windows) => "statically linked".to_string(),
        None => "dynamically linked".to_string(),
    }
}

/// Path of the loaded `lib<name>.so` / `lib<name>.dylib`, when the platform lets us look.
fn loaded_shared_object(lib: &str) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let needle = format!("lib{lib}.so");
        let maps = std::fs::read_to_string("/proc/self/maps").ok()?;
        return maps
            .lines()
            .filter_map(|l| l.split_whitespace().nth(5))
            .find(|p| {
                std::path::Path::new(p)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .is_some_and(|f| f.starts_with(&needle))
            })
            .map(str::to_string);
    }
    #[cfg(target_os = "macos")]
    {
        let needle = format!("lib{lib}.");
        // SAFETY: dyld image APIs are read-only and the index stays below the count we just read.
        unsafe {
            let count = libc::_dyld_image_count();
            for i in 0..count {
                let name = libc::_dyld_get_image_name(i);
                if name.is_null() {
                    continue;
                }
                let name = std::ffi::CStr::from_ptr(name).to_string_lossy();
                let file = name.rsplit('/').next().unwrap_or("");
                if file.starts_with(&needle) && file.contains(".dylib") {
                    return Some(name.into_owned());
                }
            }
        }
        return None;
    }
    #[allow(unreachable_code)]
    {
        let _ = lib;
        None
    }
}

/// libxml2's version, as compiled against (`LIBXML_DOTTED_VERSION` from the headers, e.g. `2.13.9`).
/// On the dynamic-linking platforms this is the version of the OS library at build time.
fn libxml2_version() -> String {
    let bytes = libxml::bindings::LIBXML_DOTTED_VERSION;
    std::ffi::CStr::from_bytes_until_nul(bytes)
        .map(|c| c.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "?".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_report_has_every_line() {
        let text = long();
        assert!(text.starts_with(&format!("gURL {} (", env!("CARGO_PKG_VERSION"))));
        assert!(text.contains(&format!("\nhurl {HURL_VERSION} (")));
        assert!(text.contains(" libcurl/"));
        assert!(text.contains("\nFeatures (libcurl):  "));
        assert!(text.contains("\nlibcurl:  "));
        assert!(text.contains("\nlibxml2:  2."));
    }
}
