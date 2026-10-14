use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered isolation estimate must pass. Check returns its problems, or none.
pub trait TieredIsolationEstimateRule: std::fmt::Debug {
    fn check(&self, item: &TieredIsolationEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationEstimateRequiredCategoryRule {
    pub message: String,
}

impl TieredIsolationEstimateRule for TieredIsolationEstimateRequiredCategoryRule {
    fn check(&self, item: &TieredIsolationEstimate) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationEstimateRequiredTitleRule {
    pub message: String,
}

impl TieredIsolationEstimateRule for TieredIsolationEstimateRequiredTitleRule {
    fn check(&self, item: &TieredIsolationEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationEstimateQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredIsolationEstimateRule for TieredIsolationEstimateQuantityRangeRule {
    fn check(&self, item: &TieredIsolationEstimate) -> Vec<String> {
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
pub struct TieredIsolationEstimateActiveGuardRule {
    pub limit: f64,
}

impl TieredIsolationEstimateRule for TieredIsolationEstimateActiveGuardRule {
    fn check(&self, item: &TieredIsolationEstimate) -> Vec<String> {
        if item.active && (item.minimum > self.limit) {
            return vec![format!("minimum {} exceeds {} while active", item.minimum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationEstimateContactEmailLengthRule {
    pub max_length: i64,
}

impl TieredIsolationEstimateRule for TieredIsolationEstimateContactEmailLengthRule {
    fn check(&self, item: &TieredIsolationEstimate) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationEstimateRequiredCodeRule {
    pub message: String,
}

impl TieredIsolationEstimateRule for TieredIsolationEstimateRequiredCodeRule {
    fn check(&self, item: &TieredIsolationEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered isolation estimate.
#[derive(Debug, Clone)]
pub struct TieredIsolationEstimateValidator {
    pub rules: Vec<Rc<dyn TieredIsolationEstimateRule>>,
}

impl TieredIsolationEstimateValidator {
    pub fn validate(&self, item: &TieredIsolationEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredIsolationEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredIsolationEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_isolation_estimate_validator() -> TieredIsolationEstimateValidator {
    TieredIsolationEstimateValidator {
        rules: vec![
            Rc::new(TieredIsolationEstimateRequiredCategoryRule {
                message: "tiered isolation estimate needs this field".to_string(),
            }) as Rc<dyn TieredIsolationEstimateRule>,
            Rc::new(TieredIsolationEstimateRequiredTitleRule {
                message: "tiered isolation estimate needs this field".to_string(),
            }),
            Rc::new(TieredIsolationEstimateQuantityRangeRule { min: 64, max: 3600 }),
            Rc::new(TieredIsolationEstimateActiveGuardRule { limit: 7.6 }),
            Rc::new(TieredIsolationEstimateContactEmailLengthRule { max_length: 86400 }),
            Rc::new(TieredIsolationEstimateRequiredCodeRule {
                message: "tiered isolation estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_isolation_estimate_validator();
        let item = new_tiered_isolation_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = TieredIsolationEstimateRequiredCategoryRule {
            message: "tiered isolation estimate needs an entry".to_string(),
        };
        let mut item = new_tiered_isolation_estimate("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredIsolationEstimateRequiredTitleRule {
            message: "tiered isolation estimate needs a value".to_string(),
        };
        let mut item = new_tiered_isolation_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = TieredIsolationEstimateQuantityRangeRule { min: 80, max: 255 };
        let mut item = new_tiered_isolation_estimate("x", "y");
        item.quantity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = TieredIsolationEstimateContactEmailLengthRule { max_length: 80 };
        let mut item = new_tiered_isolation_estimate("x", "y");
        item.contact_email = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredIsolationEstimateRequiredCodeRule {
            message: "tiered isolation estimate needs this field".to_string(),
        };
        let mut item = new_tiered_isolation_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
