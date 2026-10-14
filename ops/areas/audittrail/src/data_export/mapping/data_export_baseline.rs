use super::*;

use acme_kit::text;

/// Where a data export baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DataExportBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DataExportBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One data export baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportBaseline {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub port: i64,
    pub source: String,
    pub budget: f64,
    pub status: DataExportBaselineStatus,
}

impl DataExportBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportBaselineStatus) -> bool {
        match self.status {
            DataExportBaselineStatus::Open => {
                return next == DataExportBaselineStatus::InReview;
            }
            DataExportBaselineStatus::InReview => {
                match next {
                    DataExportBaselineStatus::Resolved | DataExportBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportBaselineStatus::Resolved => {
                return next == DataExportBaselineStatus::Archived;
            }
            DataExportBaselineStatus::Reopened => {
                match next {
                    DataExportBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.port as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportBaseline {
        DataExportBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            port: self.port,
            source: self.source.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a data export baseline in status open.
pub fn new_data_export_baseline(code: &str, title: &str) -> DataExportBaseline {
    DataExportBaseline {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        port: 0,
        source: "".to_string(),
        budget: 0.0,
        status: DataExportBaselineStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_data_export_baseline("code-80", "title-625");
        assert_eq!(item.status, DataExportBaselineStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_data_export_baseline("code-80", "title-625");
        if item.move_to(DataExportBaselineStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportBaselineStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_data_export_baseline("code-80", "title-625");
        item.status = DataExportBaselineStatus::Archived;
        assert!(
            item.move_to(DataExportBaselineStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
