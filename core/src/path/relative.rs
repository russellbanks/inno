use std::{
    fmt, mem,
    path::{Component, Path, PathBuf},
};

use thiserror::Error;

use super::{InnoPath, ResolveConstant, Token, resolve::is_invalid_file_name_char};

/// A relative path that is safe to join onto a directory, returned by
/// [`InnoPath::to_relative_path`].
///
/// It has at least one component, and each one is a valid Windows file name
/// other than `.` or `..`, so joining it onto a directory can only ever name
/// something inside that directory.
///
/// Its [`Display`](fmt::Display) implementation separates components with
/// `/` on every platform.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RelativePath(PathBuf);

impl RelativePath {
    /// Returns the path.
    #[must_use]
    #[inline]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Returns `base` with this path joined onto it.
    #[must_use]
    pub fn join_onto<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        base.as_ref().join(&self.0)
    }

    /// Converts this into a [`PathBuf`].
    #[must_use]
    #[inline]
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

impl AsRef<Path> for RelativePath {
    #[inline]
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl From<RelativePath> for PathBuf {
    #[inline]
    fn from(path: RelativePath) -> Self {
        path.0
    }
}

impl fmt::Display for RelativePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, component) in self.0.components().enumerate() {
            if index > 0 {
                f.write_str("/")?;
            }
            // Every component was built from a `str`, and is a normal one.
            if let Component::Normal(name) = component {
                f.write_str(&name.to_string_lossy())?;
            }
        }
        Ok(())
    }
}

/// Why an [`InnoPath`] could not be turned into a [`RelativePath`].
#[derive(Clone, Debug, Error, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum PathError {
    /// The path has no components once `.` and empty components are dropped.
    #[error("the path is empty")]
    Empty,
    /// The path starts with a separator or a drive, such as `\x` or `C:\x`.
    #[error("the path is absolute")]
    Absolute,
    /// The path has a `..` component.
    #[error("the path leaves its directory with `..`")]
    EscapesRoot,
    /// The resolver did not resolve this constant.
    #[error("the constant {{{0}}} could not be resolved")]
    UnresolvedConstant(String),
    /// This component is not a valid Windows file name: it has a character
    /// that is not allowed, is made only of dots and spaces, or is a
    /// reserved device name such as `CON` or `NUL.txt`.
    #[error("`{0}` is not a valid file name")]
    InvalidName(String),
}

/// Builds the relative path for [`InnoPath::to_relative_path`].
pub(super) fn build<R>(path: InnoPath<'_>, resolver: &R) -> Result<RelativePath, PathError>
where
    R: ResolveConstant + ?Sized,
{
    if path.as_str().starts_with(['\\', '/']) {
        return Err(PathError::Absolute);
    }

    let mut builder = Builder::default();
    for token in path.tokens() {
        match token {
            Token::Literal(text) => builder.current.push_str(text),
            Token::Separator => builder.end_component()?,
            Token::Constant(constant) => {
                let text = resolver
                    .resolve(constant)
                    .ok_or_else(|| PathError::UnresolvedConstant(constant.as_str().to_owned()))?;
                let mut parts = text.split(['\\', '/']);
                builder.current.push_str(parts.next().unwrap_or_default());
                for part in parts {
                    builder.end_component()?;
                    builder.current.push_str(part);
                }
            }
        }
    }
    builder.end_component()?;

    if builder.path.as_os_str().is_empty() {
        Err(PathError::Empty)
    } else {
        Ok(RelativePath(builder.path))
    }
}

#[derive(Default)]
struct Builder {
    path: PathBuf,
    /// The component being built, since a component can be made of several
    /// tokens, such as `foo-{cm:Lang}.txt`.
    current: String,
}

impl Builder {
    /// Checks the component built so far and adds it to the path.
    fn end_component(&mut self) -> Result<(), PathError> {
        let component = mem::take(&mut self.current);
        match component.as_str() {
            "" | "." => return Ok(()),
            ".." => return Err(PathError::EscapesRoot),
            _ => {}
        }

        // Windows ignores trailing dots and spaces, so `foo.` and `foo` are the same file.
        let name = component.trim_end_matches(['.', ' ']);

        if name.is_empty() {
            return Err(PathError::InvalidName(component));
        }
        if self.path.as_os_str().is_empty() && is_drive(name) {
            return Err(PathError::Absolute);
        }
        if name.contains(is_invalid_file_name_char) || is_reserved_name(name) {
            return Err(PathError::InvalidName(component));
        }

        self.path.push(name);
        Ok(())
    }
}

/// Returns `true` if `name` starts with a drive, such as `C:`.
fn is_drive(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// Returns `true` if `name` refers to a device on Windows, such as `CON`,
/// `nul.txt`, or `COM1`, whatever its extension.
fn is_reserved_name(name: &str) -> bool {
    const RESERVED: [&str; 4] = ["CON", "PRN", "AUX", "NUL"];

    let stem = name
        .split_once('.')
        .map_or(name, |(stem, _)| stem)
        .trim_end_matches(' ');

    if RESERVED
        .iter()
        .any(|reserved| stem.eq_ignore_ascii_case(reserved))
    {
        return true;
    }

    // COM1-COM9 and LPT1-LPT9, including the superscript digits Windows also accepts.
    let Some((prefix, digit)) = stem
        .char_indices()
        .nth(3)
        .map(|(index, _)| stem.split_at(index))
    else {
        return false;
    };
    (prefix.eq_ignore_ascii_case("COM") || prefix.eq_ignore_ascii_case("LPT"))
        && matches!(
            digit,
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
        )
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, path::PathBuf};

    use rstest::rstest;

    use super::PathError;
    use crate::path::{ByName, InnoPath, from_fn};

    fn relative(raw: &str) -> Result<String, PathError> {
        InnoPath::new(raw)
            .to_relative_path(ByName)
            .map(|path| path.to_string())
    }

    #[rstest]
    #[case::app(r"{app}\bin\foo.dll", "app/bin/foo.dll")]
    #[case::sys(r"{sys}\foo.dll", "sys/foo.dll")]
    #[case::constant_mid_path(r"{app}\{code:SubDir}\foo.dll", "app/code_SubDir/foo.dll")]
    #[case::constant_with_separators(
        r"{reg:HKLM\Software\Foo,Path|C:\x}\foo.dll",
        "reg_HKLM_Software_Foo,Path_C__x/foo.dll"
    )]
    #[case::nested_constant(r"{code:GetDir|{app}}\foo.dll", "code_GetDir_{app}/foo.dll")]
    #[case::constant_inside_name(r"{app}\foo-{cm:Lang}.txt", "app/foo-cm_Lang.txt")]
    #[case::escaped_brace(r"{app}\{{literal}.txt", "app/{literal}.txt")]
    #[case::unterminated_brace(r"{app}\{oops\foo.dll", "app/{oops/foo.dll")]
    #[case::plain(r"plain\path.txt", "plain/path.txt")]
    #[case::forward_slashes("{app}/bin/foo.dll", "app/bin/foo.dll")]
    #[case::current_dir(r"{app}\.\bin\.\foo.dll", "app/bin/foo.dll")]
    #[case::repeated_separators(r"{app}\\bin\\foo.dll", "app/bin/foo.dll")]
    #[case::trailing_separator(r"{app}\bin\", "app/bin")]
    #[case::trailing_dots_and_spaces(r"{app}\foo. .\bar.txt. ", "app/foo/bar.txt")]
    #[case::dots_in_name(r"{app}\file..name.txt", "app/file..name.txt")]
    #[case::reserved_lookalikes(r"{app}\CONSOLE\COM10\LPT\nul_", "app/CONSOLE/COM10/LPT/nul_")]
    #[case::empty_constant(r"{}\foo.dll", "foo.dll")]
    #[case::non_ascii(r"{app}\café\日本.txt", "app/café/日本.txt")]
    fn keeps_paths_inside_root(#[case] raw: &str, #[case] expected: &str) {
        assert_eq!(relative(raw).as_deref(), Ok(expected));
    }

    #[rstest]
    #[case::empty("", PathError::Empty)]
    #[case::only_empty_constant("{}", PathError::Empty)]
    #[case::only_current_dir(r".\.", PathError::Empty)]
    #[case::root(r"\", PathError::Absolute)]
    #[case::leading_separator(r"\etc\passwd", PathError::Absolute)]
    #[case::unc(r"\\server\share\x", PathError::Absolute)]
    #[case::absolute_drive(r"C:\Windows\evil.dll", PathError::Absolute)]
    #[case::drive_relative("C:evil.dll", PathError::Absolute)]
    #[case::parent_dir("..", PathError::EscapesRoot)]
    #[case::parent_after_constant(r"{app}\..\..\evil.dll", PathError::EscapesRoot)]
    #[case::parent_constant(r"{..}\evil.dll", PathError::EscapesRoot)]
    #[case::forward_parent("{app}/../evil.dll", PathError::EscapesRoot)]
    #[case::alternate_data_stream(
        r"{app}\foo.txt:stream",
        PathError::InvalidName("foo.txt:stream".into())
    )]
    #[case::wildcard(r"{app}\*.tmp", PathError::InvalidName("*.tmp".into()))]
    #[case::dots_and_spaces(r"{app}\. .\x", PathError::InvalidName(". .".into()))]
    #[case::parent_with_trailing_space(r"{app}\.. \x", PathError::InvalidName(".. ".into()))]
    #[case::reserved(r"{app}\CON", PathError::InvalidName("CON".into()))]
    #[case::reserved_with_extension(r"{app}\nul.txt", PathError::InvalidName("nul.txt".into()))]
    #[case::reserved_port(r"{app}\com1.log", PathError::InvalidName("com1.log".into()))]
    #[case::reserved_superscript(r"{app}\LPT¹", PathError::InvalidName("LPT¹".into()))]
    #[case::reserved_constant(r"{aux}\x", PathError::InvalidName("aux".into()))]
    fn rejects_unsafe_paths(#[case] raw: &str, #[case] expected: PathError) {
        assert_eq!(relative(raw), Err(expected));
    }

    #[test]
    fn custom_resolver_can_flatten_expand_and_reject() {
        let resolver = from_fn(|constant| match constant.name() {
            "app" => Some(Cow::Borrowed("")),
            "group" => Some(Cow::Borrowed(r"Start Menu\Programs")),
            "tmp" => None,
            "evil" => Some(Cow::Borrowed(r"..\..")),
            _ => Some(Cow::Borrowed(constant.as_str())),
        });
        let relative = |raw| {
            InnoPath::new(raw)
                .to_relative_path(&resolver)
                .map(|path| path.to_string())
        };

        assert_eq!(relative(r"{app}\bin\foo.dll").as_deref(), Ok("bin/foo.dll"));
        assert_eq!(
            relative(r"{group}\App.lnk").as_deref(),
            Ok("Start Menu/Programs/App.lnk")
        );
        assert_eq!(
            relative(r"{tmp}\x.dll"),
            Err(PathError::UnresolvedConstant("tmp".into()))
        );
        assert_eq!(relative(r"{evil}\x.dll"), Err(PathError::EscapesRoot));
        assert_eq!(relative(r"{app}"), Err(PathError::Empty));
    }

    #[test]
    fn joins_onto_base() {
        let path = InnoPath::new(r"{app}\bin\foo.dll")
            .to_relative_path(ByName)
            .unwrap();
        let expected: PathBuf = ["out", "app", "bin", "foo.dll"].iter().collect();
        assert_eq!(path.join_onto("out"), expected);
    }

    #[test]
    fn error_messages() {
        assert_eq!(
            PathError::UnresolvedConstant("tmp".into()).to_string(),
            "the constant {tmp} could not be resolved"
        );
        assert_eq!(
            PathError::InvalidName("CON".into()).to_string(),
            "`CON` is not a valid file name"
        );
    }
}
