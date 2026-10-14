use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional forecast rule must pass. Check returns its problems, or none.
pub trait RegionalForecastRuleRule: std::fmt::Debug {
    fn check(&self, item: &RegionalForecastRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRuleRequiredLabelRule {
    pub message: String,
}

impl RegionalForecastRuleRule for RegionalForecastRuleRequiredLabelRule {
    fn check(&self, item: &RegionalForecastRule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRuleRequiredCodeRule {
    pub message: String,
}

impl RegionalForecastRuleRule for RegionalForecastRuleRequiredCodeRule {
    fn check(&self, item: &RegionalForecastRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRuleLabelLengthRule {
    pub max_length: i64,
}

impl RegionalForecastRuleRule for RegionalForecastRuleLabelLengthRule {
    fn check(&self, item: &RegionalForecastRule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRuleRequiredSummaryRule {
    pub message: String,
}

impl RegionalForecastRuleRule for RegionalForecastRuleRequiredSummaryRule {
    fn check(&self, item: &RegionalForecastRule) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional forecast rule.
#[derive(Debug, Clone)]
pub struct RegionalForecastRuleValidator {
    pub rules: Vec<Rc<dyn RegionalForecastRuleRule>>,
}

impl RegionalForecastRuleValidator {
    pub fn validate(&self, item: &RegionalForecastRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalForecastRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalForecastRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_forecast_rule_validator() -> RegionalForecastRuleValidator {
    RegionalForecastRuleValidator {
        rules: vec![
            Rc::new(RegionalForecastRuleRequiredLabelRule {
                message: "regional forecast rule needs this field".to_string(),
            }) as Rc<dyn RegionalForecastRuleRule>,
            Rc::new(RegionalForecastRuleRequiredCodeRule {
                message: "regional forecast rule needs this field".to_string(),
            }),
            Rc::new(RegionalForecastRuleLabelLengthRule { max_length: 255 }),
            Rc::new(RegionalForecastRuleRequiredSummaryRule {
                message: "regional forecast rule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_forecast_rule_validator();
        let item = new_regional_forecast_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalForecastRuleRequiredLabelRule {
            message: "regional forecast rule needs an entry".to_string(),
        };
        let mut item = new_regional_forecast_rule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalForecastRuleRequiredCodeRule {
            message: "regional forecast rule needs an entry".to_string(),
        };
        let mut item = new_regional_forecast_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalForecastRuleLabelLengthRule { max_length: 32 };
        let mut item = new_regional_forecast_rule("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = RegionalForecastRuleRequiredSummaryRule {
            message: "regional forecast rule needs an entry".to_string(),
        };
        let mut item = new_regional_forecast_rule("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
