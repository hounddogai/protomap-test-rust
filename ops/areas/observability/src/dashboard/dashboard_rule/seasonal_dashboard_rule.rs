use super::*;

use acme_kit::text;

/// Where a seasonal dashboard rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDashboardRuleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalDashboardRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalDashboardRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal dashboard rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardRule {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub margin: f64,
    pub locale: String,
    pub external_ref: String,
    pub score: f64,
    pub status: SeasonalDashboardRuleStatus,
}

impl SeasonalDashboardRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDashboardRuleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDashboardRuleStatus) -> bool {
        match self.status {
            SeasonalDashboardRuleStatus::Created => {
                return next == SeasonalDashboardRuleStatus::Validated || next == SeasonalDashboardRuleStatus::Retired;
            }
            SeasonalDashboardRuleStatus::Validated => {
                return next == SeasonalDashboardRuleStatus::Retired;
            }
            SeasonalDashboardRuleStatus::Published => {
                match next {
                    SeasonalDashboardRuleStatus::Retired => {
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

    pub fn move_to(&mut self, next: SeasonalDashboardRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal dashboard rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.window_days as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDashboardRule {
        SeasonalDashboardRule {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            margin: self.margin,
            locale: self.locale.clone(),
            external_ref: self.external_ref.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a seasonal dashboard rule in status created.
pub fn new_seasonal_dashboard_rule(code: &str, label: &str) -> SeasonalDashboardRule {
    SeasonalDashboardRule {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        margin: 0.0,
        locale: "".to_string(),
        external_ref: "".to_string(),
        score: 0.0,
        status: SeasonalDashboardRuleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_seasonal_dashboard_rule("code-701", "label-22");
        assert_eq!(item.status, SeasonalDashboardRuleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_seasonal_dashboard_rule("code-701", "label-22");
        if item.move_to(SeasonalDashboardRuleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalDashboardRuleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_seasonal_dashboard_rule("code-701", "label-22");
        item.status = SeasonalDashboardRuleStatus::Retired;
        assert!(
            item.move_to(SeasonalDashboardRuleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
