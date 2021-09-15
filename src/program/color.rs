use image::Rgb;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Color {
    White,
    Black,

    LightRed,
    LightYellow,
    LightGreen,
    LightCyan,
    LightBlue,
    LightMagenta,

    Red,
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,

    DarkRed,
    DarkYellow,
    DarkGreen,
    DarkCyan,
    DarkBlue,
    DarkMagenta,
}

impl Color {
    #[allow(dead_code)]
    pub fn to_rgb8(self) -> Rgb<u8> {
        use Color::*;

        #[rustfmt::skip]
        match self {
            LightRed     => Rgb([0xFF, 0xC0, 0xC0]),
            LightYellow  => Rgb([0xFF, 0xFF, 0xC0]),
            LightGreen   => Rgb([0xC0, 0xFF, 0xC0]),
            LightCyan    => Rgb([0xC0, 0xFF, 0xFF]),
            LightBlue    => Rgb([0xC0, 0xC0, 0xFF]),
            LightMagenta => Rgb([0xFF, 0xC0, 0xFF]),

            Red          => Rgb([0xFF, 0x00, 0x00]),
            Yellow       => Rgb([0xFF, 0xFF, 0x00]),
            Green        => Rgb([0x00, 0xFF, 0x00]),
            Cyan         => Rgb([0x00, 0xFF, 0xFF]),
            Blue         => Rgb([0x00, 0x00, 0xFF]),
            Magenta      => Rgb([0xFF, 0x00, 0xFF]),

            DarkRed      => Rgb([0xC0, 0x00, 0x00]),
            DarkYellow   => Rgb([0xC0, 0xC0, 0x00]),
            DarkGreen    => Rgb([0x00, 0xC0, 0x00]),
            DarkCyan     => Rgb([0x00, 0xC0, 0xC0]),
            DarkBlue     => Rgb([0x00, 0x00, 0xC0]),
            DarkMagenta  => Rgb([0xC0, 0x00, 0xC0]),

            White        => Rgb([0xFF, 0xFF, 0xFF]),
            Black        => Rgb([0x00, 0x00, 0x00]),
        }
    }

    pub fn from_rgb8(rgb: &Rgb<u8>) -> Self {
        use Color::*;

        match rgb {
            Rgb([0xFF, 0xC0, 0xC0]) => LightRed,
            Rgb([0xFF, 0xFF, 0xC0]) => LightYellow,
            Rgb([0xC0, 0xFF, 0xC0]) => LightGreen,
            Rgb([0xC0, 0xFF, 0xFF]) => LightCyan,
            Rgb([0xC0, 0xC0, 0xFF]) => LightBlue,
            Rgb([0xFF, 0xC0, 0xFF]) => LightMagenta,

            Rgb([0xFF, 0x00, 0x00]) => Red,
            Rgb([0xFF, 0xFF, 0x00]) => Yellow,
            Rgb([0x00, 0xFF, 0x00]) => Green,
            Rgb([0x00, 0xFF, 0xFF]) => Cyan,
            Rgb([0x00, 0x00, 0xFF]) => Blue,
            Rgb([0xFF, 0x00, 0xFF]) => Magenta,

            Rgb([0xC0, 0x00, 0x00]) => DarkRed,
            Rgb([0xC0, 0xC0, 0x00]) => DarkYellow,
            Rgb([0x00, 0xC0, 0x00]) => DarkGreen,
            Rgb([0x00, 0xC0, 0xC0]) => DarkCyan,
            Rgb([0x00, 0x00, 0xC0]) => DarkBlue,
            Rgb([0xC0, 0x00, 0xC0]) => DarkMagenta,

            Rgb([0xFF, 0xFF, 0xFF]) => White,
            Rgb([0x00, 0x00, 0x00]) => Black,

            // If the colour is not matched we can interpret it as white
            Rgb(_) => {
                log::warn!("Encountered an unrecognised colour: {:?}", rgb);
                White
            }
        }
    }

    fn hue_number(&self) -> Option<i32> {
        use Color::*;

        #[rustfmt::skip]
        match *self {
            LightRed     | Red     | DarkRed     => Some(0),
            LightYellow  | Yellow  | DarkYellow  => Some(1),
            LightGreen   | Green   | DarkGreen   => Some(2),
            LightCyan    | Cyan    | DarkCyan    => Some(3),
            LightBlue    | Blue    | DarkBlue    => Some(4),
            LightMagenta | Magenta | DarkMagenta => Some(5),

            White | Black => None,
        }
    }

    pub fn hue_change(&self, other: &Self) -> Option<u32> {
        let n1 = self.hue_number()?;
        let n2 = other.hue_number()?;

        Some((n2 - n1).rem_euclid(6) as u32)
    }

    fn lightness_number(&self) -> Option<i32> {
        use Color::*;

        #[rustfmt::skip]
        match *self {
            LightRed | LightYellow | LightGreen | LightCyan | LightBlue | LightMagenta => Some(0),
            Red      | Yellow      | Green      | Cyan      | Blue      | Magenta      => Some(1),
            DarkRed  | DarkYellow  | DarkGreen  | DarkCyan  | DarkBlue  | DarkMagenta  => Some(2),

            White | Black => None,
        }
    }

    pub fn lightness_change(&self, other: &Self) -> Option<u32> {
        let n1 = self.lightness_number()?;
        let n2 = other.lightness_number()?;

        Some((n2 - n1).rem_euclid(3) as u32)
    }
}