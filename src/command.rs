// Shamefully taken from https://github.com/EdgewaterDevelopment/rust-ssd1331

use super::interface::DisplayInterface;

/// SSD1331 Commands

/// Commands
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
    /// Set horizontal or vertical direction swap and color mode
    RemapAndColorDepth(bool, bool, ColorMode),
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
}

impl Command {
    /// Send command to SSD1331
    pub fn send<DI>(self, iface: &mut DI) -> Result<(), ()>
    where
        DI: DisplayInterface,
    {
        // Transform command into a fixed size array of 7 u8 and the real length for sending
        let (data, len) = match self {
            Command::Contrast(a, b, c) => ([0x81, a, 0x82, b, 0x83, c, 0], 6),
            Command::AllOn(on) => ([0xA4 | (on as u8), 0, 0, 0, 0, 0, 0], 1),
            Command::Invert(inv) => ([if inv { 0xA7 } else { 0xA4 }, 0, 0, 0, 0, 0, 0], 1),
            Command::DisplayOn(on) => ([0xAE | (on as u8), 0, 0, 0, 0, 0, 0], 1),
            Command::ColumnAddress(start, end) => ([0x15, start, end, 0, 0, 0, 0], 3),
            Command::RowAddress(start, end) => ([0x75, start, end, 0, 0, 0, 0], 3),
            Command::StartLine(line) => ([0xA1 | (0x3F & line), 0, 0, 0, 0, 0, 0], 1),
            Command::RemapAndColorDepth(hremap, vremap, cmode) => (
                [
                    0xA0,
                    0x20 | ((vremap as u8) << 4 | (hremap as u8) << 1 | (cmode as u8) << 6),
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::Multiplex(ratio) => ([0xA8, ratio, 0, 0, 0, 0, 0], 2),
            Command::ReverseComDir(rev) => ([0xC0 | ((rev as u8) << 3), 0, 0, 0, 0, 0, 0], 1),
            Command::DisplayOffset(offset) => ([0xD3, offset, 0, 0, 0, 0, 0], 2),
            Command::ComPinConfig(alt, lr) => (
                [
                    0xDA,
                    0x2 | ((alt as u8) << 4) | ((lr as u8) << 5),
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::DisplayClockDiv(fosc, div) => {
                ([0xB3, ((0xF & fosc) << 4) | (0xF & div), 0, 0, 0, 0, 0], 2)
            }
            Command::PreChargePeriod(phase1, phase2) => (
                [0x3e, ((0xF & phase2) << 4) | (0xF & phase1), 0, 0, 0, 0, 0],
                2,
            ),
            Command::VcomhDeselect(level) => ([0xBE, (level as u8) << 1, 0, 0, 0, 0, 0], 2),
            Command::Noop => ([0xE3, 0, 0, 0, 0, 0, 0], 1),
        };

        // Send command over the interface
        iface.send_commands(&data[0..len])?;

        Ok(())
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
