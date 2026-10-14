use super::*;

use acme_kit::text;

/// Where a legacy data export review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyDataExportReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyDataExportReviewStatus {
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

impl std::fmt::Display for LegacyDataExportReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy data export review as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportReview {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub contact_email: String,
    pub version: i64,
    pub tenant_id: String,
    pub revision: i64,
    pub ends_at: i64,
    pub status: LegacyDataExportReviewStatus,
}

impl LegacyDataExportReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyDataExportReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyDataExportReviewStatus) -> bool {
        match self.status {
            LegacyDataExportReviewStatus::Open => {
                return next == LegacyDataExportReviewStatus::Archived;
            }
            LegacyDataExportReviewStatus::InReview => {
                return next == LegacyDataExportReviewStatus::Resolved;
            }
            LegacyDataExportReviewStatus::Resolved => {
                return next == LegacyDataExportReviewStatus::Reopened;
            }
            LegacyDataExportReviewStatus::Reopened => {
                return next == LegacyDataExportReviewStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyDataExportReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy data export review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy data export review", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.rotation_days as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyDataExportReview {
        LegacyDataExportReview {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            contact_email: self.contact_email.clone(),
            version: self.version,
            tenant_id: self.tenant_id.clone(),
            revision: self.revision,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy data export review in status open.
pub fn new_legacy_data_export_review(code: &str, name: &str) -> LegacyDataExportReview {
    LegacyDataExportReview {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        contact_email: "".to_string(),
        version: 0,
        tenant_id: "".to_string(),
        revision: 0,
        ends_at: 0,
        status: LegacyDataExportReviewStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_data_export_review("code-400", "name-427");
        assert_eq!(item.status, LegacyDataExportReviewStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_legacy_data_export_review("code-400", "name-427");
        if item.move_to(LegacyDataExportReviewStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyDataExportReviewStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_data_export_review("code-400", "name-427");
        item.status = LegacyDataExportReviewStatus::Archived;
        assert!(
            item.move_to(LegacyDataExportReviewStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
