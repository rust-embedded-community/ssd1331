# Changelog

`ssd1331` is a Rust driver for the SSD1331 OLED display driver/module. It implements the
`embedded-hal` traits to allow easy integration with embedded Rust projects using an SPI interface.

## Unreleased

The driver has been drastically simplified with removal of the `RawMode` and `GraphicsMode` structs, as well as the `Builder`.

[embedded-graphics](https://crates.io/crates/embedded-graphics) is also upgraded to version `0.6.0-alpha.2`.

### Migrating from 0.1 to 0.2

Version 0.1.x

```rust
use ssd1331::{prelude::*, Builder};

let mut disp: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();

disp.reset(&mut rst, &mut delay);
disp.init().unwrap();
disp.flush().unwrap();

disp.get_dimensions();
disp.get_rotation();
```

Version 0.2.x

```rust
use ssd1331::{Ssd1331, DisplayRotation};

let mut disp = Ssd1331::new(spi, dc, DisplayRotation::Rotate0);

disp.reset(&mut rst, &mut delay).unwrap();
disp.init().unwrap();
disp.flush().unwrap();

disp.dimensions();
disp.rotation();
```

### Added

- Added `ssd1331::Error` enum with pin (if DC pin fails to set) and communication error (if SPI write fails) variants. The return type of fallible methods has changed from `Result<(), ()>` to `Result<(), ssd1331::Error<CommE, PinE>>`. `CommE` and `PinE` default to `()` so no user code changes should be required.

### Changed

- **(breaking)** `display.get_dimensions()` is renamed to `display.dimensions()`
- **(breaking)** The `Builder` struct has been removed. Use `Ssd1331::new()` instead. Code that looked like this:

  ```rust
  use ssd1331::{prelude::*, Builder};

  let mut disp: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();

  disp.reset(&mut rst, &mut delay);
  disp.init().unwrap();
  disp.flush().unwrap();
  ```

  Should now look like this:

  ```rust
  use ssd1331::{Ssd1331, DisplayRotation};

  let mut disp = Ssd1331::new(spi, dc, DisplayRotation::Rotate0);

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
  use ssd1331::{Ssd1331, DisplayRotation};
  ```

  See above items about removal of `Builder` struct.

### Deprecated

- None

### Removed

- **(breaking)** Removed `RawMode` and `GraphicsMode` traits. The `.set_pixel()` and `.draw()` methods can now be used directly on the `Ssd1331` struct.
- **(breaking)** Removed `Builder` struct.

### Fixed

- None

### Security

- None
