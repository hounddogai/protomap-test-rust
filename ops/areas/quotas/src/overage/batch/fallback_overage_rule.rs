use super::*;

use acme_kit::text;

/// Where a fallback overage rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackOverageRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackOverageRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackOverageRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback overage rule as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageRule {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub ceiling: f64,
    pub created_at: i64,
    pub status: FallbackOverageRuleStatus,
}

impl FallbackOverageRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackOverageRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackOverageRuleStatus) -> bool {
        match self.status {
            FallbackOverageRuleStatus::Proposed => {
                match next {
                    FallbackOverageRuleStatus::Approved | FallbackOverageRuleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackOverageRuleStatus::Approved => {
                match next {
                    FallbackOverageRuleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackOverageRuleStatus::Rejected => {
                return next == FallbackOverageRuleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackOverageRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback overage rule {} cannot move from {} to {}",
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
        format!("{} {}", "fallback overage rule", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ratio - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackOverageRule {
        FallbackOverageRule {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            ceiling: self.ceiling,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback overage rule in status proposed.
pub fn new_fallback_overage_rule(code: &str, label: &str) -> FallbackOverageRule {
    FallbackOverageRule {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        ceiling: 0.0,
        created_at: 0,
        status: FallbackOverageRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_fallback_overage_rule("code-757", "label-472");
        assert_eq!(item.status, FallbackOverageRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_fallback_overage_rule("code-757", "label-472");
        if item.move_to(FallbackOverageRuleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackOverageRuleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_fallback_overage_rule("code-757", "label-472");
        item.status = FallbackOverageRuleStatus::Withdrawn;
        assert!(
            item.move_to(FallbackOverageRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
