use std::fmt;

use zerocopy::{FromBytes, Immutable, KnownLayout, LittleEndian, U32};

#[derive(Clone, Copy, Default, Eq, PartialEq, FromBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct Color(U32<LittleEndian>);

impl Color {
    /// Creates a new Color from a `red`, `green`, `blue` and `alpha`.
    ///
    /// # Examples
    ///
    /// ```
    /// use inno::header::Color;
    ///
    /// let color = Color::new(0xFF, 0xC8, 0xE1, 0xA);
    /// assert_eq!(color.to_string(), "#FFC8E10A");
    /// ```
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self(U32::from_bytes([alpha, blue, green, red]))
    }

    #[must_use]
    pub fn to_rgba(self) -> (u8, u8, u8, u8) {
        self.0.to_bytes().into()
    }

    /// Returns the `red` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use inno::header::Color;
    ///
    /// let color = Color::new(0x12, 0x34, 0x56, 0x78);
    /// assert_eq!(color.red(), 0x12);
    /// ```
    #[must_use]
    pub fn red(self) -> u8 {
        (self.0 >> 24).get() as u8
    }

    /// Returns the `green` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use inno::header::Color;
    ///
    /// let color = Color::new(0x12, 0x34, 0x56, 0x78);
    /// assert_eq!(color.green(), 0x34);
    /// ```
    #[must_use]
    pub fn green(self) -> u8 {
        (self.0 >> 16).get() as u8
    }

    /// Returns the `blue` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use inno::header::Color;
    ///
    /// let color = Color::new(0x12, 0x34, 0x56, 0x78);
    /// assert_eq!(color.blue(), 0x56);
    /// ```
    #[must_use]
    pub fn blue(self) -> u8 {
        (self.0 >> 8).get() as u8
    }

    /// Returns the `alpha` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use inno::header::Color;
    ///
    /// let color = Color::new(0x12, 0x34, 0x56, 0x78);
    /// assert_eq!(color.alpha(), 0x78);
    /// ```
    #[must_use]
    pub const fn alpha(self) -> u8 {
        self.0.get() as u8
    }
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (red, green, blue, alpha) = self.to_rgba();
        write!(
            f,
            "Color(red={red}, green={green}, blue={blue}, alpha={alpha}, {:#08X})",
            self.0
        )
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:08X}", self.0)
    }
}

impl From<U32<LittleEndian>> for Color {
    fn from(color: U32<LittleEndian>) -> Self {
        Self(color)
    }
}

impl From<u32> for Color {
    fn from(color: u32) -> Self {
        Self(U32::new(color))
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from((red, green, blue, alpha): (u8, u8, u8, u8)) -> Self {
        Self::new(red, green, blue, alpha)
    }
}

impl From<[u8; size_of::<u32>()]> for Color {
    fn from(rgba: [u8; size_of::<u32>()]) -> Self {
        Self(U32::from_bytes(rgba))
    }
}
