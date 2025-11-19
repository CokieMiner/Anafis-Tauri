//! Distribution analysis module

pub mod fitters;
pub mod global_optimizer;
pub mod goodness_of_fit;
pub mod moments;

pub use crate::scientific::statistics::types::*;
pub use fitters::*;
pub use global_optimizer::*;
pub use goodness_of_fit::*;
pub use moments::*;