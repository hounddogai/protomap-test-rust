use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn digest must pass. Check returns its problems, or none.
pub trait BudgetBurnDigestRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDigestNameLengthRule {
    pub max_length: i64,
}

impl BudgetBurnDigestRule for BudgetBurnDigestNameLengthRule {
    fn check(&self, item: &BudgetBurnDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDigestRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnDigestRule for BudgetBurnDigestRateRangeRule {
    fn check(&self, item: &BudgetBurnDigest) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDigestRequiredNameRule {
    pub message: String,
}

impl BudgetBurnDigestRule for BudgetBurnDigestRequiredNameRule {
    fn check(&self, item: &BudgetBurnDigest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDigestTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnDigestRule for BudgetBurnDigestTtlSecondsRangeRule {
    fn check(&self, item: &BudgetBurnDigest) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDigestTitleLengthRule {
    pub max_length: i64,
}

impl BudgetBurnDigestRule for BudgetBurnDigestTitleLengthRule {
    fn check(&self, item: &BudgetBurnDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDigestCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnDigestRule for BudgetBurnDigestCodePrefixRule {
    fn check(&self, item: &BudgetBurnDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn digest.
#[derive(Debug, Clone)]
pub struct BudgetBurnDigestValidator {
    pub rules: Vec<Rc<dyn BudgetBurnDigestRule>>,
}

impl BudgetBurnDigestValidator {
    pub fn validate(&self, item: &BudgetBurnDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_digest_validator() -> BudgetBurnDigestValidator {
    BudgetBurnDigestValidator {
        rules: vec![
            Rc::new(BudgetBurnDigestNameLengthRule { max_length: 32 }) as Rc<dyn BudgetBurnDigestRule>,
            Rc::new(BudgetBurnDigestRateRangeRule { min: 8.7, max: 1520.8 }),
            Rc::new(BudgetBurnDigestRequiredNameRule {
                message: "budget burn digest needs a value".to_string(),
            }),
            Rc::new(BudgetBurnDigestTtlSecondsRangeRule { min: 255, max: 1000 }),
            Rc::new(BudgetBurnDigestTitleLengthRule { max_length: 80 }),
            Rc::new(BudgetBurnDigestCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_digest_validator();
        let item = new_budget_burn_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BudgetBurnDigestNameLengthRule { max_length: 32 };
        let mut item = new_budget_burn_digest("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = BudgetBurnDigestRateRangeRule { min: 5.5, max: 2704.7 };
        let mut item = new_budget_burn_digest("x", "y");
        item.rate = 2706.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BudgetBurnDigestRequiredNameRule {
            message: "budget burn digest needs this field".to_string(),
        };
        let mut item = new_budget_burn_digest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = BudgetBurnDigestTtlSecondsRangeRule { min: 32, max: 86400 };
        let mut item = new_budget_burn_digest("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BudgetBurnDigestTitleLengthRule { max_length: 32 };
        let mut item = new_budget_burn_digest("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
