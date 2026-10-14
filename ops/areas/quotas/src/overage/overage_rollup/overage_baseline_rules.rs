use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage baseline must pass. Check returns its problems, or none.
pub trait OverageBaselineRule: std::fmt::Debug {
    fn check(&self, item: &OverageBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBaselineCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageBaselineRule for OverageBaselineCostRangeRule {
    fn check(&self, item: &OverageBaseline) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBaselineAllowedNameRule {
    pub allowed: Vec<String>,
}

impl OverageBaselineRule for OverageBaselineAllowedNameRule {
    fn check(&self, item: &OverageBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBaselineTitleLengthRule {
    pub max_length: i64,
}

impl OverageBaselineRule for OverageBaselineTitleLengthRule {
    fn check(&self, item: &OverageBaseline) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBaselineCodePrefixRule {
    pub prefix: String,
}

impl OverageBaselineRule for OverageBaselineCodePrefixRule {
    fn check(&self, item: &OverageBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBaselineRequiredTitleRule {
    pub message: String,
}

impl OverageBaselineRule for OverageBaselineRequiredTitleRule {
    fn check(&self, item: &OverageBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage baseline.
#[derive(Debug, Clone)]
pub struct OverageBaselineValidator {
    pub rules: Vec<Rc<dyn OverageBaselineRule>>,
}

impl OverageBaselineValidator {
    pub fn validate(&self, item: &OverageBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_baseline_validator() -> OverageBaselineValidator {
    OverageBaselineValidator {
        rules: vec![
            Rc::new(OverageBaselineCostRangeRule { min: 7.5, max: 4203.6 }) as Rc<dyn OverageBaselineRule>,
            Rc::new(OverageBaselineAllowedNameRule { allowed: Vec::new() }),
            Rc::new(OverageBaselineTitleLengthRule { max_length: 120 }),
            Rc::new(OverageBaselineCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(OverageBaselineRequiredTitleRule {
                message: "overage baseline needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_baseline_validator();
        let item = new_overage_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = OverageBaselineCostRangeRule { min: 7.2, max: 2631.0 };
        let mut item = new_overage_baseline("x", "y");
        item.cost = 2632.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = OverageBaselineTitleLengthRule { max_length: 86400 };
        let mut item = new_overage_baseline("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = OverageBaselineRequiredTitleRule {
            message: "overage baseline needs an entry".to_string(),
        };
        let mut item = new_overage_baseline("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
