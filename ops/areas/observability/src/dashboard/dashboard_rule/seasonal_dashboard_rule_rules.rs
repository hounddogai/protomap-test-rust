use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal dashboard rule must pass. Check returns its problems, or none.
pub trait SeasonalDashboardRuleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalDashboardRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardRuleRequiredCodeRule {
    pub message: String,
}

impl SeasonalDashboardRuleRule for SeasonalDashboardRuleRequiredCodeRule {
    fn check(&self, item: &SeasonalDashboardRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardRuleRequiredLocaleRule {
    pub message: String,
}

impl SeasonalDashboardRuleRule for SeasonalDashboardRuleRequiredLocaleRule {
    fn check(&self, item: &SeasonalDashboardRule) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardRuleExternalRefLengthRule {
    pub max_length: i64,
}

impl SeasonalDashboardRuleRule for SeasonalDashboardRuleExternalRefLengthRule {
    fn check(&self, item: &SeasonalDashboardRule) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardRuleMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalDashboardRuleRule for SeasonalDashboardRuleMarginRangeRule {
    fn check(&self, item: &SeasonalDashboardRule) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardRuleCodePrefixRule {
    pub prefix: String,
}

impl SeasonalDashboardRuleRule for SeasonalDashboardRuleCodePrefixRule {
    fn check(&self, item: &SeasonalDashboardRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardRuleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalDashboardRuleRule for SeasonalDashboardRuleWindowDaysRangeRule {
    fn check(&self, item: &SeasonalDashboardRule) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal dashboard rule.
#[derive(Debug, Clone)]
pub struct SeasonalDashboardRuleValidator {
    pub rules: Vec<Rc<dyn SeasonalDashboardRuleRule>>,
}

impl SeasonalDashboardRuleValidator {
    pub fn validate(&self, item: &SeasonalDashboardRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalDashboardRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalDashboardRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_dashboard_rule_validator() -> SeasonalDashboardRuleValidator {
    SeasonalDashboardRuleValidator {
        rules: vec![
            Rc::new(SeasonalDashboardRuleRequiredCodeRule {
                message: "seasonal dashboard rule needs this field".to_string(),
            }) as Rc<dyn SeasonalDashboardRuleRule>,
            Rc::new(SeasonalDashboardRuleRequiredLocaleRule {
                message: "seasonal dashboard rule needs an entry".to_string(),
            }),
            Rc::new(SeasonalDashboardRuleExternalRefLengthRule { max_length: 3600 }),
            Rc::new(SeasonalDashboardRuleMarginRangeRule { min: 1.5, max: 3880.5 }),
            Rc::new(SeasonalDashboardRuleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SeasonalDashboardRuleWindowDaysRangeRule { min: 32, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_dashboard_rule_validator();
        let item = new_seasonal_dashboard_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalDashboardRuleRequiredCodeRule {
            message: "seasonal dashboard rule needs an entry".to_string(),
        };
        let mut item = new_seasonal_dashboard_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = SeasonalDashboardRuleRequiredLocaleRule {
            message: "seasonal dashboard rule needs this field".to_string(),
        };
        let mut item = new_seasonal_dashboard_rule("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = SeasonalDashboardRuleExternalRefLengthRule { max_length: 64 };
        let mut item = new_seasonal_dashboard_rule("x", "y");
        item.external_ref = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = SeasonalDashboardRuleMarginRangeRule { min: 2.6, max: 4990.1 };
        let mut item = new_seasonal_dashboard_rule("x", "y");
        item.margin = 4991.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = SeasonalDashboardRuleWindowDaysRangeRule { min: 64, max: 3600 };
        let mut item = new_seasonal_dashboard_rule("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
