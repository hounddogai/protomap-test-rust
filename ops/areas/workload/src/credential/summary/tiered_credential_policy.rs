use super::*;

use acme_kit::text;

/// Where a tiered credential policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredCredentialPolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredCredentialPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TieredCredentialPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered credential policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialPolicy {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub category: String,
    pub margin: f64,
    pub active: bool,
    pub key_id: String,
    pub approved: bool,
    pub pinned: bool,
    pub status: TieredCredentialPolicyStatus,
}

impl TieredCredentialPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredCredentialPolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredCredentialPolicyStatus) -> bool {
        match self.status {
            TieredCredentialPolicyStatus::Planned => {
                return next == TieredCredentialPolicyStatus::InProgress
                    || next == TieredCredentialPolicyStatus::Completed;
            }
            TieredCredentialPolicyStatus::Scheduled => {
                match next {
                    TieredCredentialPolicyStatus::InProgress | TieredCredentialPolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredCredentialPolicyStatus::InProgress => {
                return next == TieredCredentialPolicyStatus::Completed
                    || next == TieredCredentialPolicyStatus::Cancelled;
            }
            TieredCredentialPolicyStatus::Completed => {
                match next {
                    TieredCredentialPolicyStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredCredentialPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered credential policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.quantity as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredCredentialPolicy {
        TieredCredentialPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            category: self.category.clone(),
            margin: self.margin,
            active: self.active,
            key_id: self.key_id.clone(),
            approved: self.approved,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tiered credential policy in status planned.
pub fn new_tiered_credential_policy(code: &str, name: &str) -> TieredCredentialPolicy {
    TieredCredentialPolicy {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        category: "".to_string(),
        margin: 0.0,
        active: false,
        key_id: "".to_string(),
        approved: false,
        pinned: false,
        status: TieredCredentialPolicyStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_credential_policy("code-924", "name-819");
        assert_eq!(item.status, TieredCredentialPolicyStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_tiered_credential_policy("code-924", "name-819");
        if item.move_to(TieredCredentialPolicyStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredCredentialPolicyStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_credential_policy("code-924", "name-819");
        item.status = TieredCredentialPolicyStatus::Cancelled;
        assert!(
            item.move_to(TieredCredentialPolicyStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
