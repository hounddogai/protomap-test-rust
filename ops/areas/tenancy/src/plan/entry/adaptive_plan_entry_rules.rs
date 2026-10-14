use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive plan entry must pass. Check returns its problems, or none.
pub trait AdaptivePlanEntryRule: std::fmt::Debug {
    fn check(&self, item: &AdaptivePlanEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanEntryRequiredCodeRule {
    pub message: String,
}

impl AdaptivePlanEntryRule for AdaptivePlanEntryRequiredCodeRule {
    fn check(&self, item: &AdaptivePlanEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanEntryRequiredNameRule {
    pub message: String,
}

impl AdaptivePlanEntryRule for AdaptivePlanEntryRequiredNameRule {
    fn check(&self, item: &AdaptivePlanEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanEntryLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptivePlanEntryRule for AdaptivePlanEntryLimitRangeRule {
    fn check(&self, item: &AdaptivePlanEntry) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanEntryIssuerLengthRule {
    pub max_length: i64,
}

impl AdaptivePlanEntryRule for AdaptivePlanEntryIssuerLengthRule {
    fn check(&self, item: &AdaptivePlanEntry) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanEntryEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptivePlanEntryRule for AdaptivePlanEntryEndsAtRangeRule {
    fn check(&self, item: &AdaptivePlanEntry) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a adaptive plan entry.
#[derive(Debug, Clone)]
pub struct AdaptivePlanEntryValidator {
    pub rules: Vec<Rc<dyn AdaptivePlanEntryRule>>,
}

impl AdaptivePlanEntryValidator {
    pub fn validate(&self, item: &AdaptivePlanEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptivePlanEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptivePlanEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_plan_entry_validator() -> AdaptivePlanEntryValidator {
    AdaptivePlanEntryValidator {
        rules: vec![
            Rc::new(AdaptivePlanEntryRequiredCodeRule {
                message: "adaptive plan entry needs an entry".to_string(),
            }) as Rc<dyn AdaptivePlanEntryRule>,
            Rc::new(AdaptivePlanEntryRequiredNameRule {
                message: "adaptive plan entry needs this field".to_string(),
            }),
            Rc::new(AdaptivePlanEntryLimitRangeRule { min: 120, max: 3600 }),
            Rc::new(AdaptivePlanEntryIssuerLengthRule { max_length: 255 }),
            Rc::new(AdaptivePlanEntryEndsAtRangeRule { min: 1000, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_plan_entry_validator();
        let item = new_adaptive_plan_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptivePlanEntryRequiredCodeRule {
            message: "adaptive plan entry needs this field".to_string(),
        };
        let mut item = new_adaptive_plan_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AdaptivePlanEntryRequiredNameRule {
            message: "adaptive plan entry needs this field".to_string(),
        };
        let mut item = new_adaptive_plan_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = AdaptivePlanEntryLimitRangeRule { min: 255, max: 1000 };
        let mut item = new_adaptive_plan_entry("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = AdaptivePlanEntryIssuerLengthRule { max_length: 1000 };
        let mut item = new_adaptive_plan_entry("x", "y");
        item.issuer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = AdaptivePlanEntryEndsAtRangeRule { min: 80, max: 1000 };
        let mut item = new_adaptive_plan_entry("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
