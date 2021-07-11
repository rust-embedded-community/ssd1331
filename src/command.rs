// Shamefully taken from https://github.com/EdgewaterDevelopment/rust-ssd1331

use crate::error::Error;
use embedded_hal::digital::v2::OutputPin;

/// SSD1331 Commands
#[derive(Debug)]
#[allow(dead_code)]
pub enum Command {
    /// Set (r, g, b) contrast. Higher number is higher contrast.
    Contrast(u8, u8, u8),
    /// Turn entire display on. If set, all pixels will
    /// be set to on, if not, the value in memory will be used.
    AllOn(bool),
    /// Invert display.
    Invert(bool),
    /// Turn display on or off.
    DisplayOn(bool),
    /// Setup column start and end address
    /// values range from 0-127
    /// This is only for horizontal or vertical addressing mode
    ColumnAddress(u8, u8),
    /// Setup row start and end address
    RowAddress(u8, u8),
    /// Set display start line from 0-63
    StartLine(u8),
    /// Set horizontal or vertical direction swap, color format/depth and address increment mode
    RemapAndColorDepth(bool, bool, ColorMode, AddressIncrementMode),
    /// Set multipex ratio from 15-63 (MUX-1)
    Multiplex(u8),
    /// Scan from COM[n-1] to COM0 (where N is mux ratio)
    ReverseComDir(bool),
    /// Set vertical shift
    DisplayOffset(u8),
    /// Setup com hardware configuration
    /// First value indicates sequential (false) or alternative (true)
    /// pin configuration. Second value disables (false) or enables (true)
    /// left/right remap.
    ComPinConfig(bool, bool),
    /// Set up display clock.
    /// First value is oscillator frequency, increasing with higher value
    /// Second value is divide ratio - 1
    DisplayClockDiv(u8, u8),
    /// Set up phase 1 and 2 of precharge period. each value is from 0-63
    PreChargePeriod(u8, u8),
    /// Set Vcomh Deselect level
    VcomhDeselect(VcomhLevel),
    /// NOOP
    Noop,
    /// Draw a line (col start, col end, row start, row end, 16-bit color)
    DrawLine(u8, u8, u8, u8, u16),
    /// Draw a rectangle (col start, col end, row start, row end, line color, fill color)
    /// Note: fill color will have no use if EnableFill was sent with false
    DrawRect(u8, u8, u8, u8, u16, u16),
    /// Enable filling of drawn rectangles
    EnableFill(bool),
}

/// This is a raw converter from Rgb565 u16 to the bytes that
/// ssd1331 expects for rgb in accelerated graphics commands.
fn raw16_to_ssd1331_accel(raw: u16) -> (u8, u8, u8) {
    const RED_MASK: u16 = 0b11111_000000_00000;
    const GREEN_MASK: u16 = 0b00000_111111_00000;
    const BLUE_MASK: u16 = 0b00000_000000_11111;
    let a = ((raw & BLUE_MASK) << 1) as u8;
    let b = ((raw & GREEN_MASK) >> 5) as u8;
    let c = ((raw & RED_MASK) >> 10) as u8;
    (a, b, c)
}

impl Command {
    /// Send command to SSD1331
    pub fn send<SPI, DC, CommE, PinE>(
        self,
        spi: &mut SPI,
        dc: &mut DC,
    ) -> Result<(), Error<CommE, PinE>>
    where
        SPI: hal::blocking::spi::Write<u8, Error = CommE>,
        DC: OutputPin<Error = PinE>,
    {
        // Transform command into a fixed size array of 11 u8 and the real length for sending
        let (data, len) = match self {
            Command::Contrast(a, b, c) => ([0x81, a, 0x82, b, 0x83, c, 0, 0, 0, 0, 0], 6),
            // TODO: Collapse AllOn and Invert commands into new DisplayMode cmd with enum
            Command::AllOn(on) => ([if on { 0xA5 } else { 0xA6 }, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            Command::Invert(inv) => ([if inv { 0xA7 } else { 0xA4 }, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            Command::DisplayOn(on) => ([0xAE | (on as u8), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            Command::ColumnAddress(start, end) => ([0x15, start, end, 0, 0, 0, 0, 0, 0, 0, 0], 3),
            Command::RowAddress(start, end) => ([0x75, start, end, 0, 0, 0, 0, 0, 0, 0, 0], 3),
            Command::StartLine(line) => ([0xA1, (0x3F & line), 0, 0, 0, 0, 0, 0, 0, 0, 0], 2),
            Command::RemapAndColorDepth(hremap, vremap, cmode, addr_inc_mode) => (
                [
                    0xA0,
                    0x20 | ((vremap as u8) << 4
                        | (hremap as u8) << 1
                        | (cmode as u8) << 6
                        | (addr_inc_mode as u8)),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::Multiplex(ratio) => ([0xA8, ratio, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2),
            Command::ReverseComDir(rev) => ([0xC0 | ((rev as u8) << 3), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            Command::DisplayOffset(offset) => ([0xA2, offset, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2),
            Command::ComPinConfig(alt, lr) => (
                [
                    0xDA,
                    0x2 | ((alt as u8) << 4) | ((lr as u8) << 5),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::DisplayClockDiv(fosc, div) => {
                ([0xB3, ((0xF & fosc) << 4) | (0xF & div), 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::PreChargePeriod(phase1, phase2) => (
                [0x3e, ((0xF & phase2) << 4) | (0xF & phase1), 0, 0, 0, 0, 0, 0, 0, 0, 0],
                2,
            ),
            Command::VcomhDeselect(level) => ([0xBE, (level as u8) << 1, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2),
            Command::Noop => ([0xE3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            Command::DrawLine(c1, r1, c2, r2, color_raw16) => {
                // do it by hand since graphics is an optional feature
                let (a, b, c) = raw16_to_ssd1331_accel(color_raw16);
                ([0x21, c1, r1, c2, r2, c, b, a, 0, 0, 0], 8)
            },
            Command::DrawRect(c1, r1, c2, r2, line_raw16, fill_raw16) => {
                let (al, bl, cl) = raw16_to_ssd1331_accel(line_raw16);
                let (af, bf, cf) = raw16_to_ssd1331_accel(fill_raw16);
                ([0x22, c1, r1, c2, r2, cl, bl, al, cf, bf, af], 11)
            },
            Command::EnableFill(on) => ([0x26, if on { 0x01 } else { 0x00 }, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2),
        };

        // Command mode. 1 = data, 0 = command
        dc.set_low().map_err(Error::Pin)?;

        // Send command over the interface
        spi.write(&data[0..len]).map_err(Error::Comm)
    }
}

/// Horizontal Scroll Direction
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum HScrollDir {
    /// Left to right
    LeftToRight = 0,
    /// Right to left
    RightToLeft = 1,
}

/// Vertical and horizontal scroll dir
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum VHScrollDir {
    /// Vertical and right horizontal
    VerticalRight = 0b01,
    /// Vertical and left horizontal
    VerticalLeft = 0b10,
}

/// Frame interval
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum NFrames {
    /// 2 Frames
    F2 = 0b111,
    /// 3 Frames
    F3 = 0b100,
    /// 4 Frames
    F4 = 0b101,
    /// 5 Frames
    F5 = 0b000,
    /// 25 Frames
    F25 = 0b110,
    /// 64 Frames
    F64 = 0b001,
    /// 128 Frames
    F128 = 0b010,
    /// 256 Frames
    F256 = 0b011,
}

/// Vcomh Deselect level
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum VcomhLevel {
    /// 0.44 * Vcc
    V044 = 0b00000,
    /// 0.52 * Vcc
    V052 = 0b01000,
    /// 0.61 * Vcc
    V061 = 0b10000,
    /// 0.71 * Vcc
    V071 = 0b11000,
    /// 0.83 * Vcc
    V083 = 0b11111,
}

/// Color mode
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum ColorMode {
    /// 256 colors per pixel
    CM256 = 0x00,

    /// 65k colors per pixel
    CM65k = 0x01,
}

/// Address increment mode
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum AddressIncrementMode {
    /// Horizontal address increment
    Horizontal = 0x00,

    /// Vertical address increment
    Vertical = 0x01,
}
