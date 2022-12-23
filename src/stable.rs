//! Implementation of both sync and unsync 'stable' cells - The cross between an `Rc` and a
//! `RefCell`, which has single ownership but references may outlive the originating cell.
//!
//! This allows for mutable references to the contained data, unlike an `Rc`.

mod util;
pub mod cell;
pub mod lock;
