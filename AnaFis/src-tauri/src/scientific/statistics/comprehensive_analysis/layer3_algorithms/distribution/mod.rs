//! Distribution analysis module

pub mod fitting;
pub mod fitters;
pub mod goodness_of_fit;
pub mod moments;

pub use crate::scientific::statistics::types::*;
pub use fitting::*;
pub use fitters::*;
pub use goodness_of_fit::*;
pub use moments::*;