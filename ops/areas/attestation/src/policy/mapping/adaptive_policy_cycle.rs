use super::*;

use acme_kit::text;

/// Where a adaptive policy cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptivePolicyCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptivePolicyCycleStatus {
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

impl std::fmt::Display for AdaptivePolicyCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive policy cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePolicyCycle {
    pub code: String,
    pub label: String,
    pub category: String,
    pub margin: f64,
    pub discount: f64,
    pub attempts: i64,
    pub channel: String,
    pub tenant_id: String,
    pub approved: bool,
    pub status: AdaptivePolicyCycleStatus,
}

impl AdaptivePolicyCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptivePolicyCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptivePolicyCycleStatus) -> bool {
        match self.status {
            AdaptivePolicyCycleStatus::Draft => {
                return next == AdaptivePolicyCycleStatus::Closed;
            }
            AdaptivePolicyCycleStatus::Pending => {
                return next == AdaptivePolicyCycleStatus::Suspended || next == AdaptivePolicyCycleStatus::Closed;
            }
            AdaptivePolicyCycleStatus::Active => {
                return next == AdaptivePolicyCycleStatus::Suspended || next == AdaptivePolicyCycleStatus::Closed;
            }
            AdaptivePolicyCycleStatus::Suspended => {
                match next {
                    AdaptivePolicyCycleStatus::Closed => {
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

    pub fn move_to(&mut self, next: AdaptivePolicyCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive policy cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.attempts as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptivePolicyCycle {
        AdaptivePolicyCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            margin: self.margin,
            discount: self.discount,
            attempts: self.attempts,
            channel: self.channel.clone(),
            tenant_id: self.tenant_id.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a adaptive policy cycle in status draft.
pub fn new_adaptive_policy_cycle(code: &str, label: &str) -> AdaptivePolicyCycle {
    AdaptivePolicyCycle {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        margin: 0.0,
        discount: 0.0,
        attempts: 0,
        channel: "".to_string(),
        tenant_id: "".to_string(),
        approved: false,
        status: AdaptivePolicyCycleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_policy_cycle("code-131", "label-31");
        assert_eq!(item.status, AdaptivePolicyCycleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_adaptive_policy_cycle("code-131", "label-31");
        if item.move_to(AdaptivePolicyCycleStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptivePolicyCycleStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_policy_cycle("code-131", "label-31");
        item.status = AdaptivePolicyCycleStatus::Closed;
        assert!(
            item.move_to(AdaptivePolicyCycleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
