use std::borrow::Cow;

use super::Constant;

/// Decides what each constant in an [`InnoPath`] becomes when it is turned
/// into a filesystem path by [`InnoPath::to_relative_path`].
///
/// The text returned may contain `\` or `/` to expand to more than one
/// directory, or be empty to drop the constant from the path. Whatever is
/// returned is still checked, so a resolver cannot make a path escape the
/// directory it is joined onto.
///
/// [`ByName`] is the usual choice. Use [`from_fn`] to write one inline.
///
/// [`InnoPath`]: super::InnoPath
/// [`InnoPath::to_relative_path`]: super::InnoPath::to_relative_path
pub trait ResolveConstant {
    /// Returns the text that `constant` expands to, or `None` to reject any
    /// path that uses it.
    fn resolve<'c>(&self, constant: Constant<'c>) -> Option<Cow<'c, str>>;
}

impl<T: ResolveConstant + ?Sized> ResolveConstant for &T {
    #[inline]
    fn resolve<'c>(&self, constant: Constant<'c>) -> Option<Cow<'c, str>> {
        (**self).resolve(constant)
    }
}

/// Expands each constant to a directory named after it, so files installed
/// under different constants do not collide: `{app}\foo.dll` becomes
/// `app/foo.dll` and `{sys}\foo.dll` becomes `sys/foo.dll`.
///
/// The whole constant is used, so `{code:GetDir}` and `{code:GetOtherDir}`
/// also stay apart. Characters that Windows does not allow in a file name,
/// including any separators, are replaced with `_`, so a constant always
/// expands to text within a single path component:
/// `{reg:HKLM\Software\Foo,Path}` becomes `reg_HKLM_Software_Foo,Path`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct ByName;

impl ResolveConstant for ByName {
    fn resolve<'c>(&self, constant: Constant<'c>) -> Option<Cow<'c, str>> {
        Some(replace_invalid_chars(constant.as_str()))
    }
}

/// A [`ResolveConstant`] that calls a closure, returned by [`from_fn`].
#[derive(Clone, Copy, Debug)]
pub struct FromFn<F>(F);

/// Returns a [`ResolveConstant`] that calls `f` for each constant.
///
/// # Examples
///
/// Put files installed to `{app}` at the root, and reject any path that uses
/// `{tmp}`:
///
/// ```
/// use std::borrow::Cow;
///
/// use inno::path::{ByName, InnoPath, PathError, ResolveConstant, from_fn};
///
/// let resolver = from_fn(|constant| match constant.name() {
///     "app" => Some(Cow::Borrowed("")),
///     "tmp" => None,
///     _ => ByName.resolve(constant),
/// });
///
/// let path = InnoPath::new(r"{app}\bin\foo.dll").to_relative_path(&resolver)?;
/// assert_eq!(path.to_string(), "bin/foo.dll");
///
/// let path = InnoPath::new(r"{sys}\foo.dll").to_relative_path(&resolver)?;
/// assert_eq!(path.to_string(), "sys/foo.dll");
///
/// assert!(matches!(
///     InnoPath::new(r"{tmp}\foo.dll").to_relative_path(&resolver),
///     Err(PathError::UnresolvedConstant(_))
/// ));
/// # Ok::<(), PathError>(())
/// ```
pub const fn from_fn<F>(f: F) -> FromFn<F>
where
    F: for<'c> Fn(Constant<'c>) -> Option<Cow<'c, str>>,
{
    FromFn(f)
}

impl<F> ResolveConstant for FromFn<F>
where
    F: for<'c> Fn(Constant<'c>) -> Option<Cow<'c, str>>,
{
    #[inline]
    fn resolve<'c>(&self, constant: Constant<'c>) -> Option<Cow<'c, str>> {
        (self.0)(constant)
    }
}

/// Characters that Windows does not allow in a file name.
pub(super) const INVALID_FILE_NAME_CHARS: [char; 9] =
    ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

/// Returns `true` if Windows does not allow `char` in a file name.
pub(super) fn is_invalid_file_name_char(char: char) -> bool {
    INVALID_FILE_NAME_CHARS.contains(&char) || char.is_control()
}

/// Replaces each character of `text` that is not allowed in a file name
/// with `_`, borrowing `text` if there are none.
fn replace_invalid_chars(text: &str) -> Cow<'_, str> {
    if text.contains(is_invalid_file_name_char) {
        Cow::Owned(
            text.chars()
                .map(|char| {
                    if is_invalid_file_name_char(char) {
                        '_'
                    } else {
                        char
                    }
                })
                .collect(),
        )
    } else {
        Cow::Borrowed(text)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use rstest::rstest;

    use super::{ByName, ResolveConstant};
    use crate::path::{Constant, InnoPath, Token};

    fn constant(raw: &str) -> Constant<'_> {
        match InnoPath::new(raw).tokens().next() {
            Some(Token::Constant(constant)) => constant,
            other => panic!("{raw} is not a constant: {other:?}"),
        }
    }

    #[rstest]
    #[case::plain("{app}", "app")]
    #[case::code("{code:GetDir}", "code_GetDir")]
    #[case::registry(
        r"{reg:HKLM\Software\Foo,Path|C:\x}",
        "reg_HKLM_Software_Foo,Path_C__x"
    )]
    #[case::nested("{code:GetDir|{app}}", "code_GetDir_{app}")]
    #[case::control_char("{a\u{1}b}", "a_b")]
    fn by_name_replaces_invalid_chars(#[case] raw: &str, #[case] expected: &str) {
        assert_eq!(ByName.resolve(constant(raw)).as_deref(), Some(expected));
    }

    #[test]
    fn by_name_borrows_when_nothing_to_replace() {
        assert!(matches!(
            ByName.resolve(constant("{app}")),
            Some(Cow::Borrowed("app"))
        ));
    }
}
