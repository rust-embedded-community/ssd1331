# Changelog

`ssd1331` is a Rust driver for the SSD1331 OLED display driver/module. It implements the
`embedded-hal` traits to allow easy integration with embedded Rust projects using an SPI interface.

## Unreleased

Description

### Added

- Added `ssd1331::Error` enum with pin (if DC pin fails to set) and communication error (if SPI write fails) variants

### Changed

- **(breaking)** `display.get_dimensions()` is renamed to `display.dimensions()`
- **(breaking)** The `Builder` now returns an `Ssd1331` instead of a `RawMode` struct. Code like this:

  ```rust
  let mut disp: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();

      disp.reset(&mut rst, &mut delay);
      disp.init().unwrap();
      disp.flush().unwrap();
  ```

  Should now look like this:

  ```rust
  let mut disp = Builder::new().connect_spi(spi, dc);

  disp.reset(&mut rst, &mut delay).unwrap();
  disp.init().unwrap();
  disp.flush().unwrap();
  ```

- **(breaking)** Crate import structure has been simplified. Imports that looked like this:

  ```rust
  use ssd1331::prelude::*;
  use ssd1331::Builder;
  ```

  Should now look like this:

  ```rust
  use ssd1331::{Builder, Ssd1331};
  ```

### Deprecated

- None

### Removed

- **(breaking)** Removed `RawMode` and `GraphicsMode` traits. The `Builder` now returns an `Ssd1331` struct which has `.set_pixel()` from `RawMode` available. If the `graphics` feature is enabled, the embedded-graphics `.draw()` method is also available.

### Fixed

- None

### Security

- None
