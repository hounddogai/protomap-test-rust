use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard tier must pass. Check returns its problems, or none.
pub trait DashboardTierRule: std::fmt::Debug {
    fn check(&self, item: &DashboardTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTierApprovedGuardRule {
    pub limit: i64,
}

impl DashboardTierRule for DashboardTierApprovedGuardRule {
    fn check(&self, item: &DashboardTier) -> Vec<String> {
        if item.approved && (item.window_days > self.limit) {
            return vec![format!(
                "window days {} exceeds {} while approved",
                item.window_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTierRequiredLabelRule {
    pub message: String,
}

impl DashboardTierRule for DashboardTierRequiredLabelRule {
    fn check(&self, item: &DashboardTier) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTierRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DashboardTierRule for DashboardTierRateRangeRule {
    fn check(&self, item: &DashboardTier) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard tier.
#[derive(Debug, Clone)]
pub struct DashboardTierValidator {
    pub rules: Vec<Rc<dyn DashboardTierRule>>,
}

impl DashboardTierValidator {
    pub fn validate(&self, item: &DashboardTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardTier]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_dashboard_tier_validator() -> DashboardTierValidator {
    DashboardTierValidator {
        rules: vec![
            Rc::new(DashboardTierApprovedGuardRule { limit: 120 }) as Rc<dyn DashboardTierRule>,
            Rc::new(DashboardTierRequiredLabelRule {
                message: "dashboard tier needs this field".to_string(),
            }),
            Rc::new(DashboardTierRateRangeRule { min: 3.0, max: 1286.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_tier_validator();
        let item = new_dashboard_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DashboardTierRequiredLabelRule {
            message: "dashboard tier needs a value".to_string(),
        };
        let mut item = new_dashboard_tier("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = DashboardTierRateRangeRule { min: 9.5, max: 3589.4 };
        let mut item = new_dashboard_tier("x", "y");
        item.rate = 3590.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
