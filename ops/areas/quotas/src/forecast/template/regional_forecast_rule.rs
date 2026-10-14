use super::*;

use acme_kit::text;

/// Where a regional forecast rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalForecastRuleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalForecastRuleStatus {
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

impl std::fmt::Display for RegionalForecastRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional forecast rule as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRule {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub summary: String,
    pub margin: f64,
    pub quantity: i64,
    pub budget: f64,
    pub active: bool,
    pub name: String,
    pub status: RegionalForecastRuleStatus,
}

impl RegionalForecastRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalForecastRuleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalForecastRuleStatus) -> bool {
        match self.status {
            RegionalForecastRuleStatus::Open => {
                return next == RegionalForecastRuleStatus::Resolved;
            }
            RegionalForecastRuleStatus::InReview => {
                return next == RegionalForecastRuleStatus::Reopened;
            }
            RegionalForecastRuleStatus::Resolved => {
                return next == RegionalForecastRuleStatus::Reopened || next == RegionalForecastRuleStatus::Archived;
            }
            RegionalForecastRuleStatus::Reopened => {
                match next {
                    RegionalForecastRuleStatus::Archived => {
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

    pub fn move_to(&mut self, next: RegionalForecastRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional forecast rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        (self.margin * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalForecastRule {
        RegionalForecastRule {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            summary: self.summary.clone(),
            margin: self.margin,
            quantity: self.quantity,
            budget: self.budget,
            active: self.active,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional forecast rule in status open.
pub fn new_regional_forecast_rule(code: &str, label: &str) -> RegionalForecastRule {
    RegionalForecastRule {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        summary: "".to_string(),
        margin: 0.0,
        quantity: 0,
        budget: 0.0,
        active: false,
        name: "".to_string(),
        status: RegionalForecastRuleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_forecast_rule("code-290", "label-779");
        assert_eq!(item.status, RegionalForecastRuleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_regional_forecast_rule("code-290", "label-779");
        if item.move_to(RegionalForecastRuleStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalForecastRuleStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_forecast_rule("code-290", "label-779");
        item.status = RegionalForecastRuleStatus::Archived;
        assert!(
            item.move_to(RegionalForecastRuleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
