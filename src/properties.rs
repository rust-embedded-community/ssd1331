//! Container to store and set display properties

use crate::command::{ColorMode, Command, VcomhLevel};
use crate::displayrotation::DisplayRotation;
use crate::interface::DisplayInterface;

/// Display properties struct
pub struct DisplayProperties<DI> {
    iface: DI,
    display_rotation: DisplayRotation,
}

impl<DI> DisplayProperties<DI>
where
    DI: DisplayInterface,
{
    /// Create new DisplayProperties instance
    pub fn new(iface: DI, display_rotation: DisplayRotation) -> DisplayProperties<DI> {
        DisplayProperties {
            iface,
            display_rotation,
        }
    }

    /// Initialise the display in column mode (i.e. a byte walks down a column of 8 pixels) with
    /// column 0 on the left and column _(display_width - 1)_ on the right.
    pub fn init_column_mode(&mut self) -> Result<(), ()> {
        // let (_, display_height) = self.get_dimensions();

        // let display_rotation = self.display_rotation;

        // Command::DisplayOn(false).send(&mut self.iface)?;
        // // Command::DisplayClockDiv(0x8, 0x0).send(&mut self.iface)?;
        // Command::Multiplex(display_height - 1).send(&mut self.iface)?;
        // Command::DisplayOffset(0).send(&mut self.iface)?;
        // Command::StartLine(0).send(&mut self.iface)?;
        // // Command::AddressMode(AddrMode::Horizontal).send(&mut self.iface)?;

        // self.set_rotation(display_rotation)?;

        // // match self.display_size {
        // //     DisplaySize::Display128x32 => Command::ComPinConfig(false, false).send(&mut self.iface),
        // //     DisplaySize::Display128x64 => Command::ComPinConfig(true, false).send(&mut self.iface),
        // //     DisplaySize::Display96x16 => Command::ComPinConfig(false, false).send(&mut self.iface),
        // // }?;

        // // Values taken from [here](https://github.com/adafruit/Adafruit-SSD1331-OLED-Driver-Library-for-Arduino/blob/master/Adafruit_SSD1331.cpp#L119-L124)
        // Command::Contrast(0x91, 0x50, 0x7D).send(&mut self.iface)?;
        // // Command::PreChargePeriod(0x1, 0xF).send(&mut self.iface)?;
        // Command::VcomhDeselect(VcomhLevel::V071).send(&mut self.iface)?;
        // Command::AllOn(false).send(&mut self.iface)?;
        // Command::Invert(false).send(&mut self.iface)?;
        // // Command::EnableScroll(false).send(&mut self.iface)?;
        // Command::DisplayOn(true).send(&mut self.iface)?;

        let cmds = [
            0xAE, 0xA0, 0x72, 0xA1, 0x0, 0xA2, 0x0, 0xA4, 0xA8, 0x3F, 0xAD, 0x8E, 0xB0, 0x0B, 0xB1,
            0x31, 0xB3, 0xF0, 0x8A, 0x64, 0x8B, 0x78, 0x8C, 0x64, 0xBB, 0x3A, 0xBE, 0x3E, 0x87,
            0x06, 0x81, 0x91, 0x82, 0x50, 0x83, 0x7D, 0xAF,
        ];

        // let cmds = [0xAF];

        self.iface.send_commands(&cmds);

        Ok(())
    }

    /// Set the position in the framebuffer of the display where any sent data should be
    /// drawn. This method can be used for changing the affected area on the screen as well
    /// as (re-)setting the start point of the next `draw` call.
    pub fn set_draw_area(&mut self, start: (u8, u8), end: (u8, u8)) -> Result<(), ()> {
        Command::ColumnAddress(start.0, end.0 - 1).send(&mut self.iface)?;
        Command::RowAddress(start.1.into(), (end.1 - 1).into()).send(&mut self.iface)?;
        Ok(())
    }

    /// Send the data to the display for drawing at the current position in the framebuffer
    /// and advance the position accordingly. Cf. `set_draw_area` to modify the affected area by
    /// this method.
    pub fn draw(&mut self, buffer: &[u8]) -> Result<(), ()> {
        self.iface.send_data(buffer)?;
        Ok(())
    }

    // TODO: Replace (u8, u8) with a dimensioney type for consistency
    // TOOD: Make doc tests work
    /// Get display dimensions, taking into account the current rotation of the display
    ///
    /// ```rust
    /// # struct FakeInterface;
    /// #
    /// # impl DisplayInterface for FakeInterface {
    /// #     fn send_command(&mut self, cmd: u8) -> Result<(), ()> { Ok(()) }
    /// #     fn send_data(&mut self, buf: &[u8]) -> Result<(), ()> { Ok(()) }
    /// # }
    /// #
    /// # let interface = FakeInterface {};
    /// #
    /// let disp = DisplayProperties::new(
    ///     interface,
    ///     DisplaySize::Display128x64,
    ///     DisplayRotation::Rotate0
    /// );
    /// assert_eq!(disp.get_dimensions(), (128, 64));
    ///
    /// # let interface = FakeInterface {};
    /// let rotated_disp = DisplayProperties::new(
    ///     interface,
    ///     DisplaySize::Display128x64,
    ///     DisplayRotation::Rotate90
    /// );
    /// assert_eq!(rotated_disp.get_dimensions(), (64, 128));
    /// ```
    pub fn get_dimensions(&self) -> (u8, u8) {
        match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (96, 64),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (64, 96),
        }
    }

    /// Get the display rotation
    pub fn get_rotation(&self) -> DisplayRotation {
        self.display_rotation
    }

    /// Set the display rotation
    pub fn set_rotation(&mut self, display_rotation: DisplayRotation) -> Result<(), ()> {
        self.display_rotation = display_rotation;

        match display_rotation {
            DisplayRotation::Rotate0 => {
                Command::RemapAndColorDepth(false, false, ColorMode::CM65k)
                    .send(&mut self.iface)?;
            }
            DisplayRotation::Rotate90 => {
                Command::RemapAndColorDepth(false, false, ColorMode::CM65k)
                    .send(&mut self.iface)?;
            }
            DisplayRotation::Rotate180 => {
                Command::RemapAndColorDepth(false, false, ColorMode::CM65k)
                    .send(&mut self.iface)?;
            }
            DisplayRotation::Rotate270 => {
                Command::RemapAndColorDepth(false, false, ColorMode::CM65k)
                    .send(&mut self.iface)?;
            }
        };

        Ok(())
    }
}
