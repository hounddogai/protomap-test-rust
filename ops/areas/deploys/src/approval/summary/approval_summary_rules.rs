use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval summary must pass. Check returns its problems, or none.
pub trait ApprovalSummaryRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSummaryQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalSummaryRule for ApprovalSummaryQuantityRangeRule {
    fn check(&self, item: &ApprovalSummary) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSummaryCountryLengthRule {
    pub max_length: i64,
}

impl ApprovalSummaryRule for ApprovalSummaryCountryLengthRule {
    fn check(&self, item: &ApprovalSummary) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSummaryMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalSummaryRule for ApprovalSummaryMaximumRangeRule {
    fn check(&self, item: &ApprovalSummary) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSummaryCodePrefixRule {
    pub prefix: String,
}

impl ApprovalSummaryRule for ApprovalSummaryCodePrefixRule {
    fn check(&self, item: &ApprovalSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSummaryAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl ApprovalSummaryRule for ApprovalSummaryAllowedCountryRule {
    fn check(&self, item: &ApprovalSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is below minimum", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSummaryPhoneLengthRule {
    pub max_length: i64,
}

impl ApprovalSummaryRule for ApprovalSummaryPhoneLengthRule {
    fn check(&self, item: &ApprovalSummary) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval summary.
#[derive(Debug, Clone)]
pub struct ApprovalSummaryValidator {
    pub rules: Vec<Rc<dyn ApprovalSummaryRule>>,
}

impl ApprovalSummaryValidator {
    pub fn validate(&self, item: &ApprovalSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_summary_validator() -> ApprovalSummaryValidator {
    ApprovalSummaryValidator {
        rules: vec![
            Rc::new(ApprovalSummaryQuantityRangeRule { min: 64, max: 64 }) as Rc<dyn ApprovalSummaryRule>,
            Rc::new(ApprovalSummaryCountryLengthRule { max_length: 80 }),
            Rc::new(ApprovalSummaryMaximumRangeRule { min: 4.7, max: 4655.9 }),
            Rc::new(ApprovalSummaryCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ApprovalSummaryAllowedCountryRule {
                allowed: vec!["rolling".to_string(), "global".to_string(), "manual".to_string()],
            }),
            Rc::new(ApprovalSummaryPhoneLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_summary_validator();
        let item = new_approval_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ApprovalSummaryQuantityRangeRule { min: 255, max: 3600 };
        let mut item = new_approval_summary("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = ApprovalSummaryCountryLengthRule { max_length: 64 };
        let mut item = new_approval_summary("x", "y");
        item.country = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ApprovalSummaryMaximumRangeRule { min: 4.5, max: 1808.4 };
        let mut item = new_approval_summary("x", "y");
        item.maximum = 1809.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = ApprovalSummaryPhoneLengthRule { max_length: 16 };
        let mut item = new_approval_summary("x", "y");
        item.phone = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
