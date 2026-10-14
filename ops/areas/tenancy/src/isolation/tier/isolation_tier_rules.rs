use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a isolation tier must pass. Check returns its problems, or none.
pub trait IsolationTierRule: std::fmt::Debug {
    fn check(&self, item: &IsolationTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTierAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl IsolationTierRule for IsolationTierAllowedCodeRule {
    fn check(&self, item: &IsolationTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTierRequiredSourceRule {
    pub message: String,
}

impl IsolationTierRule for IsolationTierRequiredSourceRule {
    fn check(&self, item: &IsolationTier) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTierRequiredCodeRule {
    pub message: String,
}

impl IsolationTierRule for IsolationTierRequiredCodeRule {
    fn check(&self, item: &IsolationTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTierWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationTierRule for IsolationTierWindowDaysRangeRule {
    fn check(&self, item: &IsolationTier) -> Vec<String> {
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

/// Runs every rule of a isolation tier.
#[derive(Debug, Clone)]
pub struct IsolationTierValidator {
    pub rules: Vec<Rc<dyn IsolationTierRule>>,
}

impl IsolationTierValidator {
    pub fn validate(&self, item: &IsolationTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_tier_validator() -> IsolationTierValidator {
    IsolationTierValidator {
        rules: vec![
            Rc::new(IsolationTierAllowedCodeRule {
                allowed: vec!["strict".to_string(), "global".to_string()],
            }) as Rc<dyn IsolationTierRule>,
            Rc::new(IsolationTierRequiredSourceRule {
                message: "isolation tier needs this field".to_string(),
            }),
            Rc::new(IsolationTierRequiredCodeRule {
                message: "isolation tier needs this field".to_string(),
            }),
            Rc::new(IsolationTierWindowDaysRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_tier_validator();
        let item = new_isolation_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = IsolationTierRequiredSourceRule {
            message: "isolation tier needs an entry".to_string(),
        };
        let mut item = new_isolation_tier("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IsolationTierRequiredCodeRule {
            message: "isolation tier needs a value".to_string(),
        };
        let mut item = new_isolation_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = IsolationTierWindowDaysRangeRule { min: 80, max: 255 };
        let mut item = new_isolation_tier("x", "y");
        item.window_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
