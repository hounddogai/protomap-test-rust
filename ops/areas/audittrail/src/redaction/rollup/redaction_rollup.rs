use super::*;

use acme_kit::text;

/// Where a redaction rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RedactionRollupStatus {
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

impl std::fmt::Display for RedactionRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRollup {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub grace_days: i64,
    pub external_ref: String,
    pub ratio: f64,
    pub status: RedactionRollupStatus,
}

impl RedactionRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionRollupStatus) -> bool {
        match self.status {
            RedactionRollupStatus::Open => {
                return next == RedactionRollupStatus::Reopened;
            }
            RedactionRollupStatus::InReview => {
                return next == RedactionRollupStatus::Resolved || next == RedactionRollupStatus::Archived;
            }
            RedactionRollupStatus::Resolved => {
                return next == RedactionRollupStatus::Reopened || next == RedactionRollupStatus::Archived;
            }
            RedactionRollupStatus::Reopened => {
                return next == RedactionRollupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "redaction rollup", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ratio + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionRollup {
        RedactionRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            grace_days: self.grace_days,
            external_ref: self.external_ref.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a redaction rollup in status open.
pub fn new_redaction_rollup(code: &str, name: &str) -> RedactionRollup {
    RedactionRollup {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        grace_days: 0,
        external_ref: "".to_string(),
        ratio: 0.0,
        status: RedactionRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_redaction_rollup("code-502", "name-676");
        assert_eq!(item.status, RedactionRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_redaction_rollup("code-502", "name-676");
        if item.move_to(RedactionRollupStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionRollupStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_redaction_rollup("code-502", "name-676");
        item.status = RedactionRollupStatus::Archived;
        assert!(
            item.move_to(RedactionRollupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
