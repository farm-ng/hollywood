/// The custom periodic actor.
pub mod periodic;
pub use periodic::Periodic;

/// Generic printer actor.
pub mod printer;
pub use printer::{Printer, PrinterProp};

/// Nudge actor.
pub mod nudge;
pub use nudge::Nudge;

/// Zip actor.
pub mod zip;
pub use zip::Zip10;
pub use zip::Zip11;
pub use zip::Zip12;
pub use zip::Zip2;
pub use zip::Zip3;
pub use zip::Zip4;
pub use zip::Zip5;
pub use zip::Zip6;
pub use zip::Zip7;
pub use zip::Zip8;
pub use zip::Zip9;

/// Egui actor.
#[cfg(feature = "egui")]
pub mod egui;
