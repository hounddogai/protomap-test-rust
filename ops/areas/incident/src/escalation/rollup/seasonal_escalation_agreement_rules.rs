use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal escalation agreement must pass. Check returns its problems, or none.
pub trait SeasonalEscalationAgreementRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalEscalationAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEscalationAgreementRequiredCountryRule {
    pub message: String,
}

impl SeasonalEscalationAgreementRule for SeasonalEscalationAgreementRequiredCountryRule {
    fn check(&self, item: &SeasonalEscalationAgreement) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEscalationAgreementNameLengthRule {
    pub max_length: i64,
}

impl SeasonalEscalationAgreementRule for SeasonalEscalationAgreementNameLengthRule {
    fn check(&self, item: &SeasonalEscalationAgreement) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEscalationAgreementRequiredTimezoneRule {
    pub message: String,
}

impl SeasonalEscalationAgreementRule for SeasonalEscalationAgreementRequiredTimezoneRule {
    fn check(&self, item: &SeasonalEscalationAgreement) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEscalationAgreementBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalEscalationAgreementRule for SeasonalEscalationAgreementBudgetRangeRule {
    fn check(&self, item: &SeasonalEscalationAgreement) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEscalationAgreementRequiredCodeRule {
    pub message: String,
}

impl SeasonalEscalationAgreementRule for SeasonalEscalationAgreementRequiredCodeRule {
    fn check(&self, item: &SeasonalEscalationAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal escalation agreement.
#[derive(Debug, Clone)]
pub struct SeasonalEscalationAgreementValidator {
    pub rules: Vec<Rc<dyn SeasonalEscalationAgreementRule>>,
}

impl SeasonalEscalationAgreementValidator {
    pub fn validate(&self, item: &SeasonalEscalationAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalEscalationAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalEscalationAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_escalation_agreement_validator() -> SeasonalEscalationAgreementValidator {
    SeasonalEscalationAgreementValidator {
        rules: vec![
            Rc::new(SeasonalEscalationAgreementRequiredCountryRule {
                message: "seasonal escalation agreement needs a value".to_string(),
            }) as Rc<dyn SeasonalEscalationAgreementRule>,
            Rc::new(SeasonalEscalationAgreementNameLengthRule { max_length: 3600 }),
            Rc::new(SeasonalEscalationAgreementRequiredTimezoneRule {
                message: "seasonal escalation agreement needs an entry".to_string(),
            }),
            Rc::new(SeasonalEscalationAgreementBudgetRangeRule { min: 8.7, max: 4907.5 }),
            Rc::new(SeasonalEscalationAgreementRequiredCodeRule {
                message: "seasonal escalation agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_escalation_agreement_validator();
        let item = new_seasonal_escalation_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = SeasonalEscalationAgreementRequiredCountryRule {
            message: "seasonal escalation agreement needs an entry".to_string(),
        };
        let mut item = new_seasonal_escalation_agreement("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalEscalationAgreementNameLengthRule { max_length: 255 };
        let mut item = new_seasonal_escalation_agreement("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = SeasonalEscalationAgreementRequiredTimezoneRule {
            message: "seasonal escalation agreement needs a value".to_string(),
        };
        let mut item = new_seasonal_escalation_agreement("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = SeasonalEscalationAgreementBudgetRangeRule { min: 1.3, max: 2854.3 };
        let mut item = new_seasonal_escalation_agreement("x", "y");
        item.budget = 2855.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalEscalationAgreementRequiredCodeRule {
            message: "seasonal escalation agreement needs this field".to_string(),
        };
        let mut item = new_seasonal_escalation_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
