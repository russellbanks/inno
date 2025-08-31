use std::{fmt, num::ParseIntError, str::FromStr};

use thiserror::Error;
use zerocopy::{FromBytes, Immutable, KnownLayout, LE, U32};

use crate::header::WizardStyle;

/// Represents the default size of Setup Wizard windows as percentages,
/// stored in **little-endian** byte order.
///
/// This corresponds to Inno Setup's [`WizardSizePercent`] setting.
///
/// Also defined in the Inno Setup source code at:
/// <https://github.com/jrsoftware/issrc/blob/is-6_5_1/Projects/Src/Shared.Struct.pas#L120>
///
/// # Format
/// - `"a,b"` → `a` is the horizontal size, `b` is the vertical size
/// - `"a"`   → shorthand for `"a,a"` (applies the same size in both directions)
///
/// # Valid values
/// - Each size must be between **100 and 150** (inclusive).
/// - `100` means *no scaling* (original size).
/// - `120` means *20% larger*.
/// - Values outside this range are considered invalid by Inno Setup.
///
/// # Defaults
/// - [`WizardStyle::Classic`] → `100,100`
/// - [`WizardStyle::Modern`]  → `120,120`
///
/// [`WizardSizePercent`]: https://jrsoftware.org/ishelp/index.php?topic=setup_wizardsizepercent
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct WizardSizePercent {
    horizontal: U32<LE>,
    vertical: U32<LE>,
}

impl WizardSizePercent {
    const MIN: u32 = 100;
    const MAX: u32 = 150;

    const DEFAULT_CLASSIC: Self = Self::new(100, 100).unwrap();
    const DEFAULT_MODERN: Self = Self::new(120, 120).unwrap();

    /// Creates a new [`WizardSizePercent`] with the given horizontal and vertical percentages.
    #[must_use]
    #[inline]
    pub const fn new(horizontal: u32, vertical: u32) -> Option<Self> {
        if horizontal < Self::MIN || horizontal > Self::MAX {
            return None;
        }

        if vertical < Self::MIN || vertical > Self::MAX {
            return None;
        }

        Some(Self {
            horizontal: U32::new(horizontal),
            vertical: U32::new(vertical),
        })
    }

    /// Returns the horizontal scaling percentage.
    #[must_use]
    #[inline]
    pub const fn horizontal(self) -> u32 {
        self.horizontal.get()
    }

    /// Returns the vertical scaling percentage.
    #[must_use]
    #[inline]
    pub const fn vertical(self) -> u32 {
        self.vertical.get()
    }

    /// Returns both percentages as a tuple `(horizontal, vertical)`.
    #[must_use]
    #[inline]
    pub const fn as_tuple(self) -> (u32, u32) {
        (self.horizontal(), self.vertical())
    }

    /// Returns the default wizard size percent for a given [`WizardStyle`].
    ///
    /// # Defaults
    /// - [`WizardStyle::Classic`] → `100,100` (no scaling; original size)
    /// - [`WizardStyle::Modern`] → `120,120` (scaled up to 120%)
    ///
    /// # Examples
    ///
    /// ```
    /// # use inno::header::{WizardSizePercent, WizardStyle};
    ///
    /// let classic_size = WizardSizePercent::default_for(WizardStyle::Classic);
    /// assert_eq!(classic_size, WizardSizePercent::new(100, 100).unwrap());
    ///
    /// let modern_size = WizardSizePercent::default_for(WizardStyle::Modern);
    /// assert_eq!(modern_size, WizardSizePercent::new(120, 120).unwrap());
    /// ```
    #[must_use]
    pub const fn default_for(wizard_style: WizardStyle) -> Self {
        match wizard_style {
            WizardStyle::Classic => Self::DEFAULT_CLASSIC,
            WizardStyle::Modern => Self::DEFAULT_MODERN,
        }
    }
}

impl fmt::Debug for WizardSizePercent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WizardSizePercent")
            .field("horizontal", &self.horizontal())
            .field("vertical", &self.vertical())
            .finish()
    }
}

impl Default for WizardSizePercent {
    fn default() -> Self {
        Self::DEFAULT_CLASSIC
    }
}

impl fmt::Display for WizardSizePercent {
    /// Writes the Wizard Size Percent in the format `a,b`, where `a` is the horizontal size, and
    /// `b` is the vertical size.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.horizontal(), self.vertical())
    }
}

impl TryFrom<(u32, u32)> for WizardSizePercent {
    type Error = WizardSizePercentError;

    #[inline]
    fn try_from((horizontal, vertical): (u32, u32)) -> Result<Self, Self::Error> {
        if horizontal < Self::MIN || vertical < Self::MIN {
            return Err(WizardSizePercentError::NegOverflow);
        } else if horizontal > Self::MAX || vertical > Self::MAX {
            return Err(WizardSizePercentError::PosOverflow);
        }

        Ok(Self {
            horizontal: U32::new(horizontal),
            vertical: U32::new(vertical),
        })
    }
}

impl From<WizardSizePercent> for (u32, u32) {
    #[inline]
    fn from(wizard_size_percent: WizardSizePercent) -> Self {
        wizard_size_percent.as_tuple()
    }
}

#[derive(Debug, Clone, Error, Eq, PartialEq)]
pub enum WizardSizePercentError {
    #[error("Wizard Size Percentage was greater than 150")]
    PosOverflow,
    #[error("Wizard Size Percentage was less than 100")]
    NegOverflow,
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

impl FromStr for WizardSizePercent {
    type Err = WizardSizePercentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s.split_once(',').unwrap_or((s, s));

        Self::try_from((x.parse()?, y.parse()?))
    }
}
