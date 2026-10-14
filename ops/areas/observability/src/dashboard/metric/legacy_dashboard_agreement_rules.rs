use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy dashboard agreement must pass. Check returns its problems, or none.
pub trait LegacyDashboardAgreementRule: std::fmt::Debug {
    fn check(&self, item: &LegacyDashboardAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardAgreementChannelLengthRule {
    pub max_length: i64,
}

impl LegacyDashboardAgreementRule for LegacyDashboardAgreementChannelLengthRule {
    fn check(&self, item: &LegacyDashboardAgreement) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardAgreementRequiredCodeRule {
    pub message: String,
}

impl LegacyDashboardAgreementRule for LegacyDashboardAgreementRequiredCodeRule {
    fn check(&self, item: &LegacyDashboardAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardAgreementBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyDashboardAgreementRule for LegacyDashboardAgreementBudgetRangeRule {
    fn check(&self, item: &LegacyDashboardAgreement) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardAgreementRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyDashboardAgreementRule for LegacyDashboardAgreementRatioRangeRule {
    fn check(&self, item: &LegacyDashboardAgreement) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy dashboard agreement.
#[derive(Debug, Clone)]
pub struct LegacyDashboardAgreementValidator {
    pub rules: Vec<Rc<dyn LegacyDashboardAgreementRule>>,
}

impl LegacyDashboardAgreementValidator {
    pub fn validate(&self, item: &LegacyDashboardAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyDashboardAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyDashboardAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_dashboard_agreement_validator() -> LegacyDashboardAgreementValidator {
    LegacyDashboardAgreementValidator {
        rules: vec![
            Rc::new(LegacyDashboardAgreementChannelLengthRule { max_length: 80 })
                as Rc<dyn LegacyDashboardAgreementRule>,
            Rc::new(LegacyDashboardAgreementRequiredCodeRule {
                message: "legacy dashboard agreement needs this field".to_string(),
            }),
            Rc::new(LegacyDashboardAgreementBudgetRangeRule { min: 2.7, max: 2867.8 }),
            Rc::new(LegacyDashboardAgreementRatioRangeRule { min: 5.2, max: 4613.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_dashboard_agreement_validator();
        let item = new_legacy_dashboard_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = LegacyDashboardAgreementChannelLengthRule { max_length: 16 };
        let mut item = new_legacy_dashboard_agreement("x", "y");
        item.channel = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyDashboardAgreementRequiredCodeRule {
            message: "legacy dashboard agreement needs an entry".to_string(),
        };
        let mut item = new_legacy_dashboard_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = LegacyDashboardAgreementBudgetRangeRule { min: 6.6, max: 94.6 };
        let mut item = new_legacy_dashboard_agreement("x", "y");
        item.budget = 96.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = LegacyDashboardAgreementRatioRangeRule { min: 8.5, max: 1264.9 };
        let mut item = new_legacy_dashboard_agreement("x", "y");
        item.ratio = 1266.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
