use super::*;

use acme_kit::text;

/// Where a default isolation record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultIsolationRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultIsolationRecordStatus {
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

impl std::fmt::Display for DefaultIsolationRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default isolation record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationRecord {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub region: String,
    pub duration_minutes: i64,
    pub tenant_id: String,
    pub quantity: i64,
    pub attempts: i64,
    pub pinned: bool,
    pub label: String,
    pub minimum: f64,
    pub status: DefaultIsolationRecordStatus,
}

impl DefaultIsolationRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultIsolationRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultIsolationRecordStatus) -> bool {
        match self.status {
            DefaultIsolationRecordStatus::Open => {
                match next {
                    DefaultIsolationRecordStatus::Resolved | DefaultIsolationRecordStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultIsolationRecordStatus::InReview => {
                match next {
                    DefaultIsolationRecordStatus::Resolved | DefaultIsolationRecordStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultIsolationRecordStatus::Resolved => {
                match next {
                    DefaultIsolationRecordStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultIsolationRecordStatus::Reopened => {
                return next == DefaultIsolationRecordStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultIsolationRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default isolation record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default isolation record", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.minimum + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultIsolationRecord {
        DefaultIsolationRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            region: self.region.clone(),
            duration_minutes: self.duration_minutes,
            tenant_id: self.tenant_id.clone(),
            quantity: self.quantity,
            attempts: self.attempts,
            pinned: self.pinned,
            label: self.label.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default isolation record in status open.
pub fn new_default_isolation_record(code: &str, title: &str) -> DefaultIsolationRecord {
    DefaultIsolationRecord {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        region: "".to_string(),
        duration_minutes: 0,
        tenant_id: "".to_string(),
        quantity: 0,
        attempts: 0,
        pinned: false,
        label: "".to_string(),
        minimum: 0.0,
        status: DefaultIsolationRecordStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_isolation_record("code-675", "title-325");
        assert_eq!(item.status, DefaultIsolationRecordStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_default_isolation_record("code-675", "title-325");
        if item.move_to(DefaultIsolationRecordStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultIsolationRecordStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_isolation_record("code-675", "title-325");
        item.status = DefaultIsolationRecordStatus::Archived;
        assert!(
            item.move_to(DefaultIsolationRecordStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
