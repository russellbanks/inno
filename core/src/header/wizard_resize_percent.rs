use std::fmt;

use zerocopy::{FromBytes, Immutable, KnownLayout, LE, U32};

#[derive(Clone, Copy, Default, Eq, PartialEq, FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct WizardResizePercent {
    x: U32<LE>,
    y: U32<LE>,
}

impl WizardResizePercent {
    /// Creates a new `WizardResizePercent` from an `X` and `Y`.
    #[must_use]
    #[inline]
    pub const fn new(x: u32, y: u32) -> Self {
        Self {
            x: U32::new(x),
            y: U32::new(y),
        }
    }

    /// Returns the `X` percent.
    #[must_use]
    #[inline]
    pub const fn x(self) -> u32 {
        self.x.get()
    }

    /// Returns the `Y` percent.
    #[must_use]
    #[inline]
    pub const fn y(self) -> u32 {
        self.y.get()
    }

    /// Returns the `X` and `Y` percentages as a tuple.
    #[must_use]
    #[inline]
    pub const fn x_y(self) -> (u32, u32) {
        (self.x(), self.y())
    }
}

impl fmt::Debug for WizardResizePercent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WizardResizePercent")
            .field("x", &self.x())
            .field("y", &self.y())
            .finish()
    }
}

impl From<(u32, u32)> for WizardResizePercent {
    #[inline]
    fn from((x, y): (u32, u32)) -> Self {
        Self::new(x, y)
    }
}

impl From<WizardResizePercent> for (u32, u32) {
    #[inline]
    fn from(wizard_resize_percent: WizardResizePercent) -> Self {
        wizard_resize_percent.x_y()
    }
}
