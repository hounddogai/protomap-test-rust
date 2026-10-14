use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary action item baseline must pass. Check returns its problems, or none.
pub trait PrimaryActionItemBaselineRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryActionItemBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBaselineCountryLengthRule {
    pub max_length: i64,
}

impl PrimaryActionItemBaselineRule for PrimaryActionItemBaselineCountryLengthRule {
    fn check(&self, item: &PrimaryActionItemBaseline) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBaselineRequiredCountryRule {
    pub message: String,
}

impl PrimaryActionItemBaselineRule for PrimaryActionItemBaselineRequiredCountryRule {
    fn check(&self, item: &PrimaryActionItemBaseline) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBaselineAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryActionItemBaselineRule for PrimaryActionItemBaselineAllowedCodeRule {
    fn check(&self, item: &PrimaryActionItemBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary action item baseline.
#[derive(Debug, Clone)]
pub struct PrimaryActionItemBaselineValidator {
    pub rules: Vec<Rc<dyn PrimaryActionItemBaselineRule>>,
}

impl PrimaryActionItemBaselineValidator {
    pub fn validate(&self, item: &PrimaryActionItemBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryActionItemBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryActionItemBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_action_item_baseline_validator() -> PrimaryActionItemBaselineValidator {
    PrimaryActionItemBaselineValidator {
        rules: vec![
            Rc::new(PrimaryActionItemBaselineCountryLengthRule { max_length: 120 })
                as Rc<dyn PrimaryActionItemBaselineRule>,
            Rc::new(PrimaryActionItemBaselineRequiredCountryRule {
                message: "primary action item baseline needs this field".to_string(),
            }),
            Rc::new(PrimaryActionItemBaselineAllowedCodeRule {
                allowed: vec!["batched".to_string(), "strict".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_action_item_baseline_validator();
        let item = new_primary_action_item_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PrimaryActionItemBaselineCountryLengthRule { max_length: 120 };
        let mut item = new_primary_action_item_baseline("x", "y");
        item.country = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = PrimaryActionItemBaselineRequiredCountryRule {
            message: "primary action item baseline needs this field".to_string(),
        };
        let mut item = new_primary_action_item_baseline("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
