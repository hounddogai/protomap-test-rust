use super::*;

use acme_kit::text;

/// Where a data export review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportReviewStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DataExportReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DataExportReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One data export review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportReview {
    pub code: String,
    pub title: String,
    pub host: String,
    pub threshold: f64,
    pub issuer: String,
    pub status: DataExportReviewStatus,
}

impl DataExportReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportReviewStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportReviewStatus) -> bool {
        match self.status {
            DataExportReviewStatus::Planned => {
                match next {
                    DataExportReviewStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportReviewStatus::Scheduled => {
                match next {
                    DataExportReviewStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportReviewStatus::InProgress => {
                match next {
                    DataExportReviewStatus::Completed | DataExportReviewStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportReviewStatus::Completed => {
                return next == DataExportReviewStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> DataExportReview {
        DataExportReview {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            threshold: self.threshold,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a data export review in status planned.
pub fn new_data_export_review(code: &str, title: &str) -> DataExportReview {
    DataExportReview {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        threshold: 0.0,
        issuer: "".to_string(),
        status: DataExportReviewStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_data_export_review("code-816", "title-827");
        assert_eq!(item.status, DataExportReviewStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_data_export_review("code-816", "title-827");
        if item.move_to(DataExportReviewStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportReviewStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_data_export_review("code-816", "title-827");
        item.status = DataExportReviewStatus::Cancelled;
        assert!(
            item.move_to(DataExportReviewStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
