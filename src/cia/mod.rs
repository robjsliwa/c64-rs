/// CIA (Complex Interface Adapter) Module
///
/// The C64 has two CIA chips:
/// - CIA1 ($DC00-$DCFF): Keyboard, joysticks, timers, and IRQ
/// - CIA2 ($DD00-$DDFF): Serial port, VIC bank selection, and NMI
pub mod base;
pub mod timer;

pub use base::{
    CiaBase, REG_CRA, REG_CRB, REG_DDRA, REG_DDRB, REG_ICR, REG_PRA, REG_PRB, REG_TAHI, REG_TALO,
    REG_TBHI, REG_TBLO,
};
pub use timer::Timer;
