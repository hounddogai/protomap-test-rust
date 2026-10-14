use super::*;

use acme_kit::text;

/// Where a legacy policy exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyPolicyExceptionStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyPolicyExceptionStatus {
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

impl std::fmt::Display for LegacyPolicyExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy policy exception as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyException {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub sequence: i64,
    pub quantity: i64,
    pub audience: String,
    pub retries: i64,
    pub ttl_seconds: i64,
    pub owner_id: String,
    pub rotation_days: i64,
    pub summary: String,
    pub status: LegacyPolicyExceptionStatus,
}

impl LegacyPolicyException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyPolicyExceptionStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyPolicyExceptionStatus) -> bool {
        match self.status {
            LegacyPolicyExceptionStatus::New => {
                return next == LegacyPolicyExceptionStatus::Done;
            }
            LegacyPolicyExceptionStatus::Queued => {
                return next == LegacyPolicyExceptionStatus::Running || next == LegacyPolicyExceptionStatus::Failed;
            }
            LegacyPolicyExceptionStatus::Running => {
                match next {
                    LegacyPolicyExceptionStatus::Done | LegacyPolicyExceptionStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPolicyExceptionStatus::Done => {
                return next == LegacyPolicyExceptionStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyPolicyExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy policy exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy policy exception", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.sequence as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyPolicyException {
        LegacyPolicyException {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            sequence: self.sequence,
            quantity: self.quantity,
            audience: self.audience.clone(),
            retries: self.retries,
            ttl_seconds: self.ttl_seconds,
            owner_id: self.owner_id.clone(),
            rotation_days: self.rotation_days,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy policy exception in status new.
pub fn new_legacy_policy_exception(code: &str, label: &str) -> LegacyPolicyException {
    LegacyPolicyException {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        sequence: 0,
        quantity: 0,
        audience: "".to_string(),
        retries: 0,
        ttl_seconds: 0,
        owner_id: "".to_string(),
        rotation_days: 0,
        summary: "".to_string(),
        status: LegacyPolicyExceptionStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_policy_exception("code-791", "label-762");
        assert_eq!(item.status, LegacyPolicyExceptionStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_legacy_policy_exception("code-791", "label-762");
        if item.move_to(LegacyPolicyExceptionStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyPolicyExceptionStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_policy_exception("code-791", "label-762");
        item.status = LegacyPolicyExceptionStatus::Failed;
        assert!(
            item.move_to(LegacyPolicyExceptionStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
