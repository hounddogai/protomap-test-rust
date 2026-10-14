use super::*;

use acme_kit::text;

/// Where a default policy tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultPolicyTierStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultPolicyTierStatus {
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

impl std::fmt::Display for DefaultPolicyTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default policy tier as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyTier {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub floor_value: f64,
    pub sequence: i64,
    pub grace_days: i64,
    pub contact_email: String,
    pub batch_size: i64,
    pub locale: String,
    pub capacity: i64,
    pub quantity: i64,
    pub status: DefaultPolicyTierStatus,
}

impl DefaultPolicyTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultPolicyTierStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultPolicyTierStatus) -> bool {
        match self.status {
            DefaultPolicyTierStatus::Planned => {
                return next == DefaultPolicyTierStatus::InProgress || next == DefaultPolicyTierStatus::Completed;
            }
            DefaultPolicyTierStatus::Scheduled => {
                return next == DefaultPolicyTierStatus::InProgress;
            }
            DefaultPolicyTierStatus::InProgress => {
                return next == DefaultPolicyTierStatus::Completed || next == DefaultPolicyTierStatus::Cancelled;
            }
            DefaultPolicyTierStatus::Completed => {
                match next {
                    DefaultPolicyTierStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DefaultPolicyTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default policy tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.batch_size as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultPolicyTier {
        DefaultPolicyTier {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            floor_value: self.floor_value,
            sequence: self.sequence,
            grace_days: self.grace_days,
            contact_email: self.contact_email.clone(),
            batch_size: self.batch_size,
            locale: self.locale.clone(),
            capacity: self.capacity,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a default policy tier in status planned.
pub fn new_default_policy_tier(code: &str, label: &str) -> DefaultPolicyTier {
    DefaultPolicyTier {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        floor_value: 0.0,
        sequence: 0,
        grace_days: 0,
        contact_email: "".to_string(),
        batch_size: 0,
        locale: "".to_string(),
        capacity: 0,
        quantity: 0,
        status: DefaultPolicyTierStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_policy_tier("code-487", "label-285");
        assert_eq!(item.status, DefaultPolicyTierStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_default_policy_tier("code-487", "label-285");
        if item.move_to(DefaultPolicyTierStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultPolicyTierStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_policy_tier("code-487", "label-285");
        item.status = DefaultPolicyTierStatus::Cancelled;
        assert!(
            item.move_to(DefaultPolicyTierStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
