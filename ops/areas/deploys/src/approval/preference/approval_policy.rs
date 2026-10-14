use super::*;

use acme_kit::text;

/// Where a approval policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalPolicyStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ApprovalPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ApprovalPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a approval policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPolicy {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub batch_size: i64,
    pub score: f64,
    pub sequence: i64,
    pub created_at: i64,
    pub window_days: i64,
    pub status: ApprovalPolicyStatus,
}

impl ApprovalPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalPolicyStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalPolicyStatus) -> bool {
        match self.status {
            ApprovalPolicyStatus::Draft => {
                return next == ApprovalPolicyStatus::Pending;
            }
            ApprovalPolicyStatus::Pending => {
                match next {
                    ApprovalPolicyStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalPolicyStatus::Active => {
                return next == ApprovalPolicyStatus::Suspended || next == ApprovalPolicyStatus::Closed;
            }
            ApprovalPolicyStatus::Suspended => {
                match next {
                    ApprovalPolicyStatus::Closed => {
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

    pub fn move_to(&mut self, next: ApprovalPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.created_at as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalPolicy {
        ApprovalPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            batch_size: self.batch_size,
            score: self.score,
            sequence: self.sequence,
            created_at: self.created_at,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a approval policy in status draft.
pub fn new_approval_policy(code: &str, label: &str) -> ApprovalPolicy {
    ApprovalPolicy {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        batch_size: 0,
        score: 0.0,
        sequence: 0,
        created_at: 0,
        window_days: 0,
        status: ApprovalPolicyStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_approval_policy("code-114", "label-763");
        assert_eq!(item.status, ApprovalPolicyStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_approval_policy("code-114", "label-763");
        if item.move_to(ApprovalPolicyStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalPolicyStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_approval_policy("code-114", "label-763");
        item.status = ApprovalPolicyStatus::Closed;
        assert!(
            item.move_to(ApprovalPolicyStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
