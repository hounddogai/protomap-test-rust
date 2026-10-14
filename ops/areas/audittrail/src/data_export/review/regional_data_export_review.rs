use super::*;

use acme_kit::text;

/// Where a regional data export review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDataExportReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalDataExportReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RegionalDataExportReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional data export review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReview {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub tags: Vec<String>,
    pub notes: String,
    pub audience: String,
    pub amount: f64,
    pub pinned: bool,
    pub contact_email: String,
    pub status: RegionalDataExportReviewStatus,
}

impl RegionalDataExportReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDataExportReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDataExportReviewStatus) -> bool {
        match self.status {
            RegionalDataExportReviewStatus::New => {
                return next == RegionalDataExportReviewStatus::Done || next == RegionalDataExportReviewStatus::Failed;
            }
            RegionalDataExportReviewStatus::Queued => {
                return next == RegionalDataExportReviewStatus::Running;
            }
            RegionalDataExportReviewStatus::Running => {
                return next == RegionalDataExportReviewStatus::Failed;
            }
            RegionalDataExportReviewStatus::Done => {
                match next {
                    RegionalDataExportReviewStatus::Failed => {
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

    pub fn move_to(&mut self, next: RegionalDataExportReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional data export review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.version as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalDataExportReview {
        RegionalDataExportReview {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            tags: self.tags.clone(),
            notes: self.notes.clone(),
            audience: self.audience.clone(),
            amount: self.amount,
            pinned: self.pinned,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional data export review in status new.
pub fn new_regional_data_export_review(code: &str, name: &str) -> RegionalDataExportReview {
    RegionalDataExportReview {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        tags: Vec::new(),
        notes: "".to_string(),
        audience: "".to_string(),
        amount: 0.0,
        pinned: false,
        contact_email: "".to_string(),
        status: RegionalDataExportReviewStatus::New,
    }
}
