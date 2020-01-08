/// Enum of errors in this crate
///
/// Both error types `CommE` and `PinE` default to `()`
#[derive(Debug)]
pub enum Error<CommE = (), PinE = ()> {
    /// Communication error
    Comm(CommE),

    /// Pin setting error
    Pin(PinE),
}
