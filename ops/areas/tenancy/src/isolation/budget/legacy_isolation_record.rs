use super::*;

use acme_kit::text;

/// Where a legacy isolation record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyIsolationRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyIsolationRecordStatus {
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

impl std::fmt::Display for LegacyIsolationRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy isolation record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationRecord {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub region: String,
    pub contact_email: String,
    pub maximum: f64,
    pub score: f64,
    pub status: LegacyIsolationRecordStatus,
}

impl LegacyIsolationRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyIsolationRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyIsolationRecordStatus) -> bool {
        match self.status {
            LegacyIsolationRecordStatus::Open => {
                match next {
                    LegacyIsolationRecordStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyIsolationRecordStatus::InReview => {
                match next {
                    LegacyIsolationRecordStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyIsolationRecordStatus::Resolved => {
                return next == LegacyIsolationRecordStatus::Archived;
            }
            LegacyIsolationRecordStatus::Reopened => {
                match next {
                    LegacyIsolationRecordStatus::Archived => {
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

    pub fn move_to(&mut self, next: LegacyIsolationRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy isolation record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy isolation record", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.priority as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyIsolationRecord {
        LegacyIsolationRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            region: self.region.clone(),
            contact_email: self.contact_email.clone(),
            maximum: self.maximum,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a legacy isolation record in status open.
pub fn new_legacy_isolation_record(code: &str, title: &str) -> LegacyIsolationRecord {
    LegacyIsolationRecord {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        region: "".to_string(),
        contact_email: "".to_string(),
        maximum: 0.0,
        score: 0.0,
        status: LegacyIsolationRecordStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_isolation_record("code-232", "title-58");
        assert_eq!(item.status, LegacyIsolationRecordStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_legacy_isolation_record("code-232", "title-58");
        if item.move_to(LegacyIsolationRecordStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyIsolationRecordStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_isolation_record("code-232", "title-58");
        item.status = LegacyIsolationRecordStatus::Archived;
        assert!(
            item.move_to(LegacyIsolationRecordStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
