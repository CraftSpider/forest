//! Handy data structures, particularly trees and occasionally graphs

#![cfg_attr(feature = "unstable", feature(unsize, coerce_unsized))]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(never_type)]

#![deny(clippy::all)]
#![deny(
    // missing_docs,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    missing_abi,
    noop_method_call,
    pointer_structural_match,
    semicolon_in_expressions_from_macros,
    unused_import_braces,
    unused_lifetimes,
    clippy::cargo,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::ptr_as_ptr,
    clippy::cloned_instead_of_copied,
    clippy::unreadable_literal,
    clippy::missing_panics_doc
)]

extern crate alloc;

pub mod object_tree;
pub mod stable;
pub(crate) mod util;
