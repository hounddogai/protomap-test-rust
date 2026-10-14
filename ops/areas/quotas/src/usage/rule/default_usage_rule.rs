use super::*;

use acme_kit::text;

/// Where a default usage rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultUsageRuleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultUsageRuleStatus {
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

impl std::fmt::Display for DefaultUsageRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default usage rule as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageRule {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub discount: f64,
    pub external_ref: String,
    pub starts_at: i64,
    pub cost: f64,
    pub capacity: i64,
    pub key_id: String,
    pub budget: f64,
    pub status: DefaultUsageRuleStatus,
}

impl DefaultUsageRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultUsageRuleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultUsageRuleStatus) -> bool {
        match self.status {
            DefaultUsageRuleStatus::Open => {
                match next {
                    DefaultUsageRuleStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultUsageRuleStatus::InReview => {
                match next {
                    DefaultUsageRuleStatus::Resolved | DefaultUsageRuleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultUsageRuleStatus::Resolved => {
                return next == DefaultUsageRuleStatus::Reopened || next == DefaultUsageRuleStatus::Archived;
            }
            DefaultUsageRuleStatus::Reopened => {
                return next == DefaultUsageRuleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultUsageRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default usage rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.cost - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultUsageRule {
        DefaultUsageRule {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            discount: self.discount,
            external_ref: self.external_ref.clone(),
            starts_at: self.starts_at,
            cost: self.cost,
            capacity: self.capacity,
            key_id: self.key_id.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default usage rule in status open.
pub fn new_default_usage_rule(code: &str, label: &str) -> DefaultUsageRule {
    DefaultUsageRule {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        discount: 0.0,
        external_ref: "".to_string(),
        starts_at: 0,
        cost: 0.0,
        capacity: 0,
        key_id: "".to_string(),
        budget: 0.0,
        status: DefaultUsageRuleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_usage_rule("code-616", "label-368");
        assert_eq!(item.status, DefaultUsageRuleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_default_usage_rule("code-616", "label-368");
        if item.move_to(DefaultUsageRuleStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultUsageRuleStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_usage_rule("code-616", "label-368");
        item.status = DefaultUsageRuleStatus::Archived;
        assert!(
            item.move_to(DefaultUsageRuleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
