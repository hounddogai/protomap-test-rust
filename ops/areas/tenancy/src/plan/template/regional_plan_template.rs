use super::*;

use acme_kit::text;

/// Where a regional plan template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPlanTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalPlanTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalPlanTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional plan template as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanTemplate {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub currency: String,
    pub score: f64,
    pub priority: i64,
    pub scope: String,
    pub issuer: String,
    pub threshold: f64,
    pub fingerprint: String,
    pub grace_days: i64,
    pub status: RegionalPlanTemplateStatus,
}

impl RegionalPlanTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPlanTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPlanTemplateStatus) -> bool {
        match self.status {
            RegionalPlanTemplateStatus::Proposed => {
                match next {
                    RegionalPlanTemplateStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalPlanTemplateStatus::Approved => {
                return next == RegionalPlanTemplateStatus::Rejected || next == RegionalPlanTemplateStatus::Withdrawn;
            }
            RegionalPlanTemplateStatus::Rejected => {
                return next == RegionalPlanTemplateStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalPlanTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional plan template {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.grace_days as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalPlanTemplate {
        RegionalPlanTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            currency: self.currency.clone(),
            score: self.score,
            priority: self.priority,
            scope: self.scope.clone(),
            issuer: self.issuer.clone(),
            threshold: self.threshold,
            fingerprint: self.fingerprint.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a regional plan template in status proposed.
pub fn new_regional_plan_template(code: &str, label: &str) -> RegionalPlanTemplate {
    RegionalPlanTemplate {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        currency: "".to_string(),
        score: 0.0,
        priority: 0,
        scope: "".to_string(),
        issuer: "".to_string(),
        threshold: 0.0,
        fingerprint: "".to_string(),
        grace_days: 0,
        status: RegionalPlanTemplateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_plan_template("code-266", "label-317");
        assert_eq!(item.status, RegionalPlanTemplateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_regional_plan_template("code-266", "label-317");
        if item.move_to(RegionalPlanTemplateStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalPlanTemplateStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_plan_template("code-266", "label-317");
        item.status = RegionalPlanTemplateStatus::Withdrawn;
        assert!(
            item.move_to(RegionalPlanTemplateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
