//! Note about parsing CSV files:
//! These files change very infrequently.
//! Even if a file is malformed CSV, we need to adapt to the illegal format.
//! Therefore parsing errors are panic!()s with as much debug info as possible.
//! Unexpected parsing errors are irrecoverable.
//! They should always be investigated and never be handled.

mod csv;
pub use csv::{CsvIter, FileIter, FromCsv};

mod stop;
pub use stop::{StopRow, ManifestStops};

