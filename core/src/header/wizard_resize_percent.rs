#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WizardResizePercent {
    x: u32,
    y: u32,
}

impl WizardResizePercent {
    /// Creates a new `WizardResizePercent` from an `X` and `Y`.
    #[must_use]
    #[inline]
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    /// Returns the `X` percent.
    #[must_use]
    #[inline]
    pub const fn x(self) -> u32 {
        self.x
    }

    /// Returns the `Y` percent.
    #[must_use]
    #[inline]
    pub const fn y(self) -> u32 {
        self.y
    }

    /// Returns the `X` and `Y` percentages as a tuple.
    #[must_use]
    #[inline]
    pub const fn x_y(self) -> (u32, u32) {
        (self.x(), self.y())
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
