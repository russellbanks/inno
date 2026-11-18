//! Scrollable tables of installer entries.
//!
//! - [`ScrollModel`](model::ScrollModel) tracks the view and selection, with no knowledge of
//!   drawing.
//! - [`TableView`] draws a table around a model and answers mouse queries from where it last
//!   drew.
//! - [`EntryTable`] is a [`Page`] that lists entries through a [`Column`] per field.

mod entries;
mod model;
mod view;
mod width;

pub use entries::{Column, EntryTable, Page};
pub use view::{TableDrag, TableView};
