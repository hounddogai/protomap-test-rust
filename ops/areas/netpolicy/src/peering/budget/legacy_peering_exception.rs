use super::*;

use acme_kit::text;

/// Where a legacy peering exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyPeeringExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyPeeringExceptionStatus {
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

impl std::fmt::Display for LegacyPeeringExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy peering exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPeeringException {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub priority: i64,
    pub key_id: String,
    pub status: LegacyPeeringExceptionStatus,
}

impl LegacyPeeringException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyPeeringExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyPeeringExceptionStatus) -> bool {
        match self.status {
            LegacyPeeringExceptionStatus::Open => {
                return next == LegacyPeeringExceptionStatus::Reopened
                    || next == LegacyPeeringExceptionStatus::Archived;
            }
            LegacyPeeringExceptionStatus::InReview => {
                match next {
                    LegacyPeeringExceptionStatus::Resolved | LegacyPeeringExceptionStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPeeringExceptionStatus::Resolved => {
                match next {
                    LegacyPeeringExceptionStatus::Reopened | LegacyPeeringExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPeeringExceptionStatus::Reopened => {
                return next == LegacyPeeringExceptionStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyPeeringExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy peering exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy peering exception", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> LegacyPeeringException {
        LegacyPeeringException {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            priority: self.priority,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy peering exception in status open.
pub fn new_legacy_peering_exception(code: &str, label: &str) -> LegacyPeeringException {
    LegacyPeeringException {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        priority: 0,
        key_id: "".to_string(),
        status: LegacyPeeringExceptionStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_peering_exception("code-558", "label-196");
        assert_eq!(item.status, LegacyPeeringExceptionStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_legacy_peering_exception("code-558", "label-196");
        if item.move_to(LegacyPeeringExceptionStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyPeeringExceptionStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_peering_exception("code-558", "label-196");
        item.status = LegacyPeeringExceptionStatus::Archived;
        assert!(
            item.move_to(LegacyPeeringExceptionStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
