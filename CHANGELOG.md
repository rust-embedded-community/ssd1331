# Changelog

[`ssd1331`](https://crates.io/crates/ssd1331) is a Rust driver for the SSD1331 OLED display
driver/module. It implements the `embedded-hal` traits to allow easy integration with embedded Rust
projects using an SPI interface.

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Changed

- **(breaking)** [#6](https://github.com/jamwaffles/ssd1331/pull/6) Upgrade to embedded-graphics 0.6.0

### Fixed

- [#4](https://github.com/jamwaffles/ssd1331/pull/4) Guard against negative pixel coordinates panicking `draw_pixel()` calls.

## [0.2.0-alpha.2]

### Changed

- **(breaking)** Upgraded to embedded-graphics 0.6.0-alpha.3

## [0.2.0-alpha.1]

The driver has been drastically simplified with removal of the `RawMode` and `GraphicsMode` structs, as well as the `Builder`.

[embedded-graphics](https://crates.io/crates/embedded-graphics) is also upgraded to version `0.6.0-alpha.2`, so there may be breaking changes around uses of embedded-graphics.

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

- Upgraded [embedded-graphics](https://crates.io/crates/embedded-graphics) to 0.6.0-alpha.2
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

### Removed

- **(breaking)** Removed `RawMode` and `GraphicsMode` traits. The `.set_pixel()` and `.draw()` methods can now be used directly on the `Ssd1331` struct.
- **(breaking)** Removed `Builder` struct.

<!-- next-url -->

[unreleased]: https://github.com/jamwaffles/ssd1331/compare/v0.2.0-alpha.2...HEAD
[0.2.0-alpha.2]: https://github.com/jamwaffles/ssd1331/compare/v0.2.0-alpha.1...v0.2.0-alpha.2
[0.2.0-alpha.1]: https://github.com/jamwaffles/ssd1331/compare/0.1.3...v0.2.0-alpha.1
