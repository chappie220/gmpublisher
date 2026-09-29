use serde::{de::Visitor, Deserialize, Serialize};
use std::{
	fmt::Debug,
	path::{Path, PathBuf},
};

pub fn canonicalize(path: PathBuf) -> PathBuf {
	dunce::canonicalize(path.clone()).unwrap_or(path)
}

#[cfg(not(target_os = "windows"))]
pub fn normalize(path: PathBuf) -> PathBuf {
	canonicalize(path)
}

#[cfg(target_os = "windows")]
pub fn normalize(path: PathBuf) -> PathBuf {
	match dunce::canonicalize(&path) {
		Ok(canonicalized) => PathBuf::from(canonicalized.to_string_lossy().to_string().replace('\\', "/")),
		Err(_) => path,
	}
}

#[derive(Clone)]
pub struct NormalizedPathBuf {
	pub normalized: PathBuf,
	path: PathBuf,
}
impl NormalizedPathBuf {
	pub fn new() -> NormalizedPathBuf {
		NormalizedPathBuf {
			path: PathBuf::new(),
			normalized: PathBuf::new(),
		}
	}
}
impl AsRef<Path> for NormalizedPathBuf {
	fn as_ref(&self) -> &Path {
		self.path.as_ref()
	}
}
impl PartialEq for NormalizedPathBuf {
	fn eq(&self, other: &Self) -> bool {
		self.path.eq(&other.path)
	}
}
impl Eq for NormalizedPathBuf {}
impl PartialOrd for NormalizedPathBuf {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.path.partial_cmp(&other.path)
	}
}
impl Ord for NormalizedPathBuf {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.path.cmp(&other.path)
	}
}
impl std::ops::Deref for NormalizedPathBuf {
	type Target = PathBuf;
	fn deref(&self) -> &Self::Target {
		&self.path
	}
}
impl From<PathBuf> for NormalizedPathBuf {
	fn from(path: PathBuf) -> Self {
		Self {
			path: path.clone(),
			normalized: normalize(path),
		}
	}
}
impl From<&PathBuf> for NormalizedPathBuf {
	fn from(path: &PathBuf) -> Self {
		let path = path.to_owned();
		Self {
			path: path.clone(),
			normalized: normalize(path),
		}
	}
}
impl From<String> for NormalizedPathBuf {
	fn from(path: String) -> Self {
		let path = PathBuf::from(path);
		Self {
			path: path.clone(),
			normalized: normalize(path),
		}
	}
}
impl From<&str> for NormalizedPathBuf {
	fn from(path: &str) -> Self {
		let path = PathBuf::from(path);
		Self {
			path: path.clone(),
			normalized: normalize(path),
		}
	}
}
impl Debug for NormalizedPathBuf {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.path.fmt(f)
	}
}

impl Serialize for NormalizedPathBuf {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.normalized.to_string_lossy())
	}
}

struct NormalizedPathBufVisitor;
impl<'de> Visitor<'de> for NormalizedPathBufVisitor {
	type Value = String;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		formatter.write_str("a string")
	}
}
impl<'de> Deserialize<'de> for NormalizedPathBuf {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Ok(NormalizedPathBuf::from(deserializer.deserialize_string(NormalizedPathBufVisitor)?))
	}
}

#[inline]
pub fn has_extension<P: AsRef<Path>, S: AsRef<str>>(path: P, extension: S) -> bool {
	path.as_ref()
		.extension()
		.map(|x| x.to_str().map(|x| x.eq_ignore_ascii_case(extension.as_ref())).unwrap_or(false))
		.unwrap_or(false)
}

fn message_dialog(title: &str, message: String) {
	#[cfg(target_os = "linux")]
	if is_kde_session() && kdialog_message(title, &message) {
		return;
	}

	// There's no window in CLI mode (e.g. when launched from a file manager's "Open With" menu)
	if *crate::cli::CLI_MODE {
		eprintln!("{}: {}", title, message);
		return;
	}

	use tauri_plugin_dialog::DialogExt;
	crate::webview!().window().dialog().message(message).title(title).show(|_| {});
}

/// XDG_CURRENT_DESKTOP is a colon-separated list, e.g. "KDE" on Plasma
#[cfg(target_os = "linux")]
fn is_kde_session() -> bool {
	std::env::var("XDG_CURRENT_DESKTOP")
		.map(|desktops| desktops.split(':').any(|desktop| desktop.eq_ignore_ascii_case("KDE")))
		.unwrap_or(false)
}

/// Shows a native KDE message box. Returns false if kdialog isn't installed.
#[cfg(target_os = "linux")]
fn kdialog_message(title: &str, message: &str) -> bool {
	use std::process::{Command, Stdio};

	match Command::new("kdialog")
		.arg("--title")
		.arg(title)
		.arg("--msgbox")
		.arg(message)
		.stdin(Stdio::null())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn()
	{
		Ok(mut child) => {
			// Reap the process once the message box is closed
			std::thread::spawn(move || child.wait());
			true
		}
		Err(_) => false,
	}
}

pub fn open<P: AsRef<Path>>(path: P) {
	let path = path.as_ref();
	if opener::open(path).is_err() {
		message_dialog("File", path.to_string_lossy().into_owned());
	}
}

pub fn open_file_location<P: AsRef<Path>>(path: P) {
	let path = dunce::canonicalize(path.as_ref()).unwrap_or_else(|_| path.as_ref().to_path_buf());

	// D-Bus calls can take a moment (e.g. if the file manager needs to be started), don't block the command thread
	#[cfg(target_os = "linux")]
	std::thread::spawn(move || {
		if reveal_linux(&path).is_err() {
			message_dialog("File Location", path.display().to_string());
		}
	});

	#[cfg(not(target_os = "linux"))]
	if let Err(_) = (|| {
		#[cfg(target_os = "windows")]
		return std::process::Command::new("explorer").arg(format!("/select,{}", path.display())).spawn();

		#[cfg(target_os = "macos")]
		return std::process::Command::new("open").arg("-R").arg(&path).spawn();

		#[allow(unreachable_code)]
		Err(std::io::Error::new(std::io::ErrorKind::Other, "Unsupported OS"))
	})() {
		message_dialog("File Location", path.display().to_string());
	}
}

#[cfg(target_os = "linux")]
fn file_uri(path: &Path) -> String {
	use std::os::unix::ffi::OsStrExt;

	let mut uri = String::from("file://");
	for &byte in path.as_os_str().as_bytes() {
		match byte {
			b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => uri.push(byte as char),
			_ => uri.push_str(&format!("%{:02X}", byte)),
		}
	}
	uri
}

/// Highlights the file in the user's file manager (Nautilus, Dolphin, Nemo, Thunar...) using the
/// org.freedesktop.FileManager1 D-Bus interface, falling back to opening the containing directory.
#[cfg(target_os = "linux")]
fn reveal_linux(path: &Path) -> Result<(), opener::OpenError> {
	use std::process::{Command, Stdio};

	// Percent-encoded, so it's safe to embed in the GVariant/dbus-send argument syntax
	let uri = file_uri(path);

	let call = |program: &str, args: &[&str]| {
		Command::new(program)
			.args(args)
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.status()
			.map(|status| status.success())
			.unwrap_or(false)
	};

	let revealed = call(
		// gdbus ships with glib2, which WebKitGTK depends on, so it's always available
		"gdbus",
		&[
			"call",
			"--session",
			"--timeout",
			"10",
			"--dest",
			"org.freedesktop.FileManager1",
			"--object-path",
			"/org/freedesktop/FileManager1",
			"--method",
			"org.freedesktop.FileManager1.ShowItems",
			&format!("['{uri}']"),
			"''",
		],
	) || call(
		"dbus-send",
		&[
			"--session",
			"--print-reply",
			"--reply-timeout=10000",
			"--dest=org.freedesktop.FileManager1",
			"--type=method_call",
			"/org/freedesktop/FileManager1",
			"org.freedesktop.FileManager1.ShowItems",
			&format!("array:string:{uri}"),
			"string:",
		],
	);

	if revealed {
		return Ok(());
	}

	let dir = if path.is_dir() { path } else { path.parent().unwrap_or(path) };
	opener::open(dir)
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
	#[test]
	fn file_uri() {
		assert_eq!(
			super::file_uri(std::path::Path::new("/home/user/My Addons/a,b\"c'd/ü.gma")),
			"file:///home/user/My%20Addons/a%2Cb%22c%27d/%C3%BC.gma"
		);
	}
}
