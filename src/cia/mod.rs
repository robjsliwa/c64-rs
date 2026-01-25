/// CIA (Complex Interface Adapter) Module
///
/// The C64 has two CIA chips:
/// - CIA1 ($DC00-$DCFF): Keyboard, joysticks, timers, and IRQ
/// - CIA2 ($DD00-$DDFF): Serial port, VIC bank selection, and NMI
pub mod base;
pub mod timer;

pub use base::{CiaBase, REG_PRB};
