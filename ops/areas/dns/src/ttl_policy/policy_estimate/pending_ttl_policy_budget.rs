use super::*;

use acme_kit::text;

/// Where a pending ttl policy budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingTtlPolicyBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingTtlPolicyBudgetStatus {
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

impl std::fmt::Display for PendingTtlPolicyBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending ttl policy budget as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyBudget {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub window_days: i64,
    pub ratio: f64,
    pub checksum: String,
    pub rotation_days: i64,
    pub key_id: String,
    pub channel: String,
    pub cipher: String,
    pub host: String,
    pub status: PendingTtlPolicyBudgetStatus,
}

impl PendingTtlPolicyBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingTtlPolicyBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingTtlPolicyBudgetStatus) -> bool {
        match self.status {
            PendingTtlPolicyBudgetStatus::New => {
                match next {
                    PendingTtlPolicyBudgetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTtlPolicyBudgetStatus::Queued => {
                match next {
                    PendingTtlPolicyBudgetStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTtlPolicyBudgetStatus::Running => {
                return next == PendingTtlPolicyBudgetStatus::Done;
            }
            PendingTtlPolicyBudgetStatus::Done => {
                return next == PendingTtlPolicyBudgetStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingTtlPolicyBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending ttl policy budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        (self.ratio * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingTtlPolicyBudget {
        PendingTtlPolicyBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            window_days: self.window_days,
            ratio: self.ratio,
            checksum: self.checksum.clone(),
            rotation_days: self.rotation_days,
            key_id: self.key_id.clone(),
            channel: self.channel.clone(),
            cipher: self.cipher.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a pending ttl policy budget in status new.
pub fn new_pending_ttl_policy_budget(code: &str, label: &str) -> PendingTtlPolicyBudget {
    PendingTtlPolicyBudget {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        window_days: 0,
        ratio: 0.0,
        checksum: "".to_string(),
        rotation_days: 0,
        key_id: "".to_string(),
        channel: "".to_string(),
        cipher: "".to_string(),
        host: "".to_string(),
        status: PendingTtlPolicyBudgetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_ttl_policy_budget("code-409", "label-899");
        assert_eq!(item.status, PendingTtlPolicyBudgetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_pending_ttl_policy_budget("code-409", "label-899");
        if item.move_to(PendingTtlPolicyBudgetStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingTtlPolicyBudgetStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_ttl_policy_budget("code-409", "label-899");
        item.status = PendingTtlPolicyBudgetStatus::Failed;
        assert!(
            item.move_to(PendingTtlPolicyBudgetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
