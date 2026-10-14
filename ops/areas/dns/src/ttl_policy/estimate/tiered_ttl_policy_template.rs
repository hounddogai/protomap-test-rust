use super::*;

use acme_kit::text;

/// Where a tiered ttl policy template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredTtlPolicyTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredTtlPolicyTemplateStatus {
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

impl std::fmt::Display for TieredTtlPolicyTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered ttl policy template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyTemplate {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub amount: f64,
    pub duration_minutes: i64,
    pub audience: String,
    pub status: TieredTtlPolicyTemplateStatus,
}

impl TieredTtlPolicyTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredTtlPolicyTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredTtlPolicyTemplateStatus) -> bool {
        match self.status {
            TieredTtlPolicyTemplateStatus::Open => {
                return next == TieredTtlPolicyTemplateStatus::Archived;
            }
            TieredTtlPolicyTemplateStatus::InReview => {
                match next {
                    TieredTtlPolicyTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredTtlPolicyTemplateStatus::Resolved => {
                return next == TieredTtlPolicyTemplateStatus::Archived;
            }
            TieredTtlPolicyTemplateStatus::Reopened => {
                match next {
                    TieredTtlPolicyTemplateStatus::Archived => {
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

    pub fn move_to(&mut self, next: TieredTtlPolicyTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered ttl policy template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered ttl policy template", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.floor_value - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredTtlPolicyTemplate {
        TieredTtlPolicyTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            amount: self.amount,
            duration_minutes: self.duration_minutes,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered ttl policy template in status open.
pub fn new_tiered_ttl_policy_template(code: &str, label: &str) -> TieredTtlPolicyTemplate {
    TieredTtlPolicyTemplate {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        amount: 0.0,
        duration_minutes: 0,
        audience: "".to_string(),
        status: TieredTtlPolicyTemplateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_ttl_policy_template("code-715", "label-346");
        assert_eq!(item.status, TieredTtlPolicyTemplateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_tiered_ttl_policy_template("code-715", "label-346");
        if item.move_to(TieredTtlPolicyTemplateStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredTtlPolicyTemplateStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_ttl_policy_template("code-715", "label-346");
        item.status = TieredTtlPolicyTemplateStatus::Archived;
        assert!(
            item.move_to(TieredTtlPolicyTemplateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
