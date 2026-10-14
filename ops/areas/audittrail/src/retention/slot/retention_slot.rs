use super::*;

use acme_kit::text;

/// Where a retention slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RetentionSlotStatus {
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

impl std::fmt::Display for RetentionSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One retention slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSlot {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub principal: String,
    pub window_days: i64,
    pub checksum: String,
    pub version: i64,
    pub created_at: i64,
    pub rotation_days: i64,
    pub status: RetentionSlotStatus,
}

impl RetentionSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionSlotStatus) -> bool {
        match self.status {
            RetentionSlotStatus::Open => {
                return next == RetentionSlotStatus::Reopened;
            }
            RetentionSlotStatus::InReview => {
                match next {
                    RetentionSlotStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionSlotStatus::Resolved => {
                match next {
                    RetentionSlotStatus::Reopened | RetentionSlotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionSlotStatus::Reopened => {
                match next {
                    RetentionSlotStatus::Archived => {
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

    pub fn move_to(&mut self, next: RetentionSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "retention slot", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.version as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionSlot {
        RetentionSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            principal: self.principal.clone(),
            window_days: self.window_days,
            checksum: self.checksum.clone(),
            version: self.version,
            created_at: self.created_at,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a retention slot in status open.
pub fn new_retention_slot(code: &str, label: &str) -> RetentionSlot {
    RetentionSlot {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        principal: "".to_string(),
        window_days: 0,
        checksum: "".to_string(),
        version: 0,
        created_at: 0,
        rotation_days: 0,
        status: RetentionSlotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_retention_slot("code-533", "label-521");
        assert_eq!(item.status, RetentionSlotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_retention_slot("code-533", "label-521");
        if item.move_to(RetentionSlotStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionSlotStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_retention_slot("code-533", "label-521");
        item.status = RetentionSlotStatus::Archived;
        assert!(
            item.move_to(RetentionSlotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
