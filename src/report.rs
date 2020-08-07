//! Helpers to create a report for faillible processes.
use serde::Serialize;

/// Each report record will be categorized with a type implementing this
/// `ReportCategory` trait.
pub trait ReportCategory: Serialize + PartialEq {}

/// A report record.
#[derive(Debug, Serialize, PartialEq)]
struct ReportRow<R: ReportCategory> {
    category: R,
    message: String,
}

/// An report is a list of report records with 2 levels of recording: warnings
/// and errors.
#[derive(Debug, Serialize)]
pub struct Report<R: ReportCategory> {
    errors: Vec<ReportRow<R>>,
    warnings: Vec<ReportRow<R>>,
}

impl<R: ReportCategory> Default for Report<R> {
    fn default() -> Self {
        Report {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl<R: ReportCategory> Report<R> {
    /// Add a warning report record.
    pub fn add_warning(&mut self, warning: String, warning_type: R) {
        let report_row = ReportRow {
            category: warning_type,
            message: warning,
        };
        if !self.warnings.contains(&report_row) {
            self.warnings.push(report_row);
        }
    }
    /// Add an error report record.
    pub fn add_error(&mut self, error: String, error_type: R) {
        let report_row = ReportRow {
            category: error_type,
            message: error,
        };
        if !self.errors.contains(&report_row) {
            self.errors.push(report_row);
        }
    }
}
