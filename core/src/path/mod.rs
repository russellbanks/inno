//! Inno Setup path templates, and turning them into safe filesystem paths.
//!
//! Fields such as a file's destination or an icon's filename are not plain
//! paths: they are templates that mix literal text with Inno Setup constants,
//! such as `{app}\bin\{code:Arch}\foo.dll`. [`InnoPath`] borrows such a
//! field and parses it on demand into [`Token`]s. [`InnoPath::to_relative_path`]
//! is the only way to get a path from one that is safe to join onto a
//! directory, and a [`ResolveConstant`] decides what each constant becomes.
//!
//! See <https://jrsoftware.org/ishelp/topic_consts.htm> for the constants
//! Inno Setup defines.

mod relative;
mod resolve;

use std::{fmt, iter::FusedIterator};

pub use relative::{PathError, RelativePath};
pub use resolve::{ByName, FromFn, ResolveConstant, from_fn};

/// An Inno Setup path template, such as `{app}\bin\{code:Arch}\foo.dll`.
///
/// This borrows the text it was read from and parses it only when asked.
/// Its [`Display`](fmt::Display) implementation writes that text unchanged.
///
/// # Examples
///
/// ```
/// use inno::path::{ByName, InnoPath, Token};
///
/// let path = InnoPath::new(r"{app}\bin\foo.dll");
///
/// assert_eq!(path.root().map(|root| root.name()), Some("app"));
/// assert_eq!(path.tokens().filter(|token| *token == Token::Separator).count(), 2);
/// assert_eq!(path.to_relative_path(ByName)?.to_string(), "app/bin/foo.dll");
/// assert_eq!(path.to_string(), r"{app}\bin\foo.dll");
/// # Ok::<(), inno::path::PathError>(())
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct InnoPath<'a> {
    raw: &'a str,
}

impl<'a> InnoPath<'a> {
    /// Wraps `raw` as a path template.
    #[must_use]
    #[inline]
    pub const fn new(raw: &'a str) -> Self {
        Self { raw }
    }

    /// Returns the template's text, exactly as it was read.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'a str {
        self.raw
    }

    /// Returns `true` if the template is empty.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Returns an iterator over the template's literal text, separators, and
    /// constants, in order.
    #[must_use]
    #[inline]
    pub const fn tokens(&self) -> Tokens<'a> {
        Tokens { rest: self.raw }
    }

    /// Returns the constant the template starts with, such as `{app}`, if it
    /// starts with one.
    #[must_use]
    pub fn root(&self) -> Option<Constant<'a>> {
        match self.tokens().next()? {
            Token::Constant(constant) => Some(constant),
            Token::Literal(_) | Token::Separator => None,
        }
    }

    /// Returns an iterator over the constants in the template, in order.
    /// Nested constants are not listed separately.
    pub fn constants(&self) -> impl Iterator<Item = Constant<'a>> + use<'a> {
        self.tokens().filter_map(|token| match token {
            Token::Constant(constant) => Some(constant),
            Token::Literal(_) | Token::Separator => None,
        })
    }

    /// Returns the template as a relative path that is safe to join onto a
    /// directory, with each constant replaced by what `resolver` returns
    /// for it.
    ///
    /// The result never escapes the directory it is joined onto. Every
    /// component is checked, including those that `resolver` returns:
    ///
    /// - `.` components and empty components are dropped.
    /// - Trailing dots and spaces are removed from each component, as
    ///   Windows would.
    /// - A template starting with a separator or a drive, such as `\x` or
    ///   `C:\x`, is [`PathError::Absolute`].
    /// - A `..` component is [`PathError::EscapesRoot`].
    /// - A component containing a character Windows does not allow in a
    ///   file name, or which is a reserved device name such as `CON` or
    ///   `NUL.txt`, is [`PathError::InvalidName`].
    ///
    /// # Errors
    ///
    /// Returns a [`PathError`] if the template is not a safe relative path,
    /// if `resolver` cannot resolve one of its constants, or if the result
    /// is empty.
    pub fn to_relative_path<R>(&self, resolver: R) -> Result<RelativePath, PathError>
    where
        R: ResolveConstant,
    {
        relative::build(*self, &resolver)
    }
}

impl fmt::Display for InnoPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.raw)
    }
}

impl AsRef<str> for InnoPath<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.raw
    }
}

impl<'a> From<&'a str> for InnoPath<'a> {
    #[inline]
    fn from(raw: &'a str) -> Self {
        Self::new(raw)
    }
}

/// One piece of an [`InnoPath`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Token<'a> {
    /// Literal text. An escaped `{{` is yielded as the literal text `{`.
    Literal(&'a str),
    /// A path separator, `\` or `/`.
    Separator,
    /// A constant, such as `{app}`.
    Constant(Constant<'a>),
}

/// An Inno Setup constant, such as `{app}`, `{code:GetDir|default}`, or
/// `{%PATH}`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Constant<'a> {
    /// The text between the braces.
    raw: &'a str,
}

impl<'a> Constant<'a> {
    /// Returns the text between the constant's braces, such as
    /// `code:GetDir|default`.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'a str {
        self.raw
    }

    /// Returns the kind of constant: the text before the first `:`, such as
    /// `app` or `code`. An environment variable constant, `{%NAME}`, is `%`.
    #[must_use]
    pub fn name(&self) -> &'a str {
        if self.raw.starts_with('%') {
            return &self.raw[..1];
        }
        self.raw.split_once(':').map_or(self.raw, |(name, _)| name)
    }

    /// Returns what follows the constant's name, such as `GetDir|default`
    /// for `{code:GetDir|default}` or `PATH` for `{%PATH}`, or `None` for a
    /// constant without one, such as `{app}`.
    #[must_use]
    pub fn argument(&self) -> Option<&'a str> {
        if let Some(argument) = self.raw.strip_prefix('%') {
            return Some(argument);
        }
        self.raw.split_once(':').map(|(_, argument)| argument)
    }
}

impl fmt::Display for Constant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}}}", self.raw)
    }
}

/// An iterator over the [`Token`]s of an [`InnoPath`], returned by
/// [`InnoPath::tokens`].
#[derive(Clone, Debug)]
pub struct Tokens<'a> {
    rest: &'a str,
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let rest = self.rest;
        let first = rest.chars().next()?;

        let (token, consumed) = match first {
            '\\' | '/' => (Token::Separator, 1),
            '{' if rest[1..].starts_with('{') => (Token::Literal(&rest[..1]), 2),
            '{' => match closing_brace(&rest[1..]) {
                Some(end) => (
                    Token::Constant(Constant {
                        raw: &rest[1..=end],
                    }),
                    end + 2,
                ),
                // An unterminated constant: keep its brace as text.
                None => (Token::Literal(&rest[..1]), 1),
            },
            _ => {
                let end = rest.find(['\\', '/', '{']).unwrap_or(rest.len());
                (Token::Literal(&rest[..end]), end)
            }
        };

        self.rest = &rest[consumed..];
        Some(token)
    }
}

impl FusedIterator for Tokens<'_> {}

/// Returns the index of the `}` that closes a constant whose contents start
/// at the beginning of `text`, skipping over any nested constants.
fn closing_brace(text: &str) -> Option<usize> {
    let mut depth = 0_usize;
    for (index, char) in text.char_indices() {
        match char {
            '{' => depth += 1,
            '}' if depth == 0 => return Some(index),
            '}' => depth -= 1,
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{Constant, InnoPath, Token};

    const fn constant(raw: &str) -> Token<'_> {
        Token::Constant(Constant { raw })
    }

    #[rstest]
    #[case::constant_and_literals(
        r"{app}\bin\foo.dll",
        &[constant("app"), Token::Separator, Token::Literal("bin"), Token::Separator, Token::Literal("foo.dll")]
    )]
    #[case::forward_slash("a/b", &[Token::Literal("a"), Token::Separator, Token::Literal("b")])]
    #[case::constant_inside_name(
        "foo-{cm:Lang}.txt",
        &[Token::Literal("foo-"), constant("cm:Lang"), Token::Literal(".txt")]
    )]
    #[case::separators_inside_constant(
        r"{reg:HKLM\Software\Foo,Path}\x",
        &[constant(r"reg:HKLM\Software\Foo,Path"), Token::Separator, Token::Literal("x")]
    )]
    #[case::nested_constant("{code:GetDir|{app}}", &[constant("code:GetDir|{app}")])]
    #[case::escaped_brace(
        "{{literal}.txt",
        &[Token::Literal("{"), Token::Literal("literal}.txt")]
    )]
    #[case::unterminated_brace(
        r"{oops\foo",
        &[Token::Literal("{"), Token::Literal("oops"), Token::Separator, Token::Literal("foo")]
    )]
    #[case::empty_constant("{}", &[constant("")])]
    #[case::empty("", &[])]
    #[case::non_ascii(
        r"{app}\café\日本.txt",
        &[constant("app"), Token::Separator, Token::Literal("café"), Token::Separator, Token::Literal("日本.txt")]
    )]
    fn tokenizes(#[case] raw: &str, #[case] expected: &[Token]) {
        assert_eq!(InnoPath::new(raw).tokens().collect::<Vec<_>>(), expected);
    }

    #[rstest]
    #[case::plain("app", "app", None)]
    #[case::code("code:GetDir|default", "code", Some("GetDir|default"))]
    #[case::nested("code:GetDir|{app}", "code", Some("GetDir|{app}"))]
    #[case::environment("%PATH|C:\\", "%", Some("PATH|C:\\"))]
    #[case::empty("", "", None)]
    fn splits_constant(#[case] raw: &str, #[case] name: &str, #[case] argument: Option<&str>) {
        let constant = Constant { raw };
        assert_eq!(constant.name(), name);
        assert_eq!(constant.argument(), argument);
        assert_eq!(constant.to_string(), format!("{{{raw}}}"));
    }

    #[test]
    fn root_is_leading_constant_only() {
        assert_eq!(
            InnoPath::new(r"{sys}\x").root().map(|root| root.name()),
            Some("sys")
        );
        assert_eq!(InnoPath::new(r"x\{sys}").root(), None);
        assert_eq!(InnoPath::new("").root(), None);
    }

    #[test]
    fn lists_constants() {
        let path = InnoPath::new(r"{app}\{code:Sub}\{{x}\foo-{cm:Lang}");
        let names: Vec<_> = path.constants().map(|constant| constant.name()).collect();
        assert_eq!(names, ["app", "code", "cm"]);
    }

    #[test]
    fn displays_raw_text() {
        let raw = r"{app}\{{x}\{code:F|{tmp}}\foo";
        assert_eq!(InnoPath::new(raw).to_string(), raw);
    }
}
