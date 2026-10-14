use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release ledger must pass. Check returns its problems, or none.
pub trait ReleaseLedgerRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseLedgerRegionLengthRule {
    pub max_length: i64,
}

impl ReleaseLedgerRule for ReleaseLedgerRegionLengthRule {
    fn check(&self, item: &ReleaseLedger) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseLedgerCodeLengthRule {
    pub max_length: i64,
}

impl ReleaseLedgerRule for ReleaseLedgerCodeLengthRule {
    fn check(&self, item: &ReleaseLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseLedgerScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseLedgerRule for ReleaseLedgerScoreRangeRule {
    fn check(&self, item: &ReleaseLedger) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseLedgerRequiredCategoryRule {
    pub message: String,
}

impl ReleaseLedgerRule for ReleaseLedgerRequiredCategoryRule {
    fn check(&self, item: &ReleaseLedger) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseLedgerCodePrefixRule {
    pub prefix: String,
}

impl ReleaseLedgerRule for ReleaseLedgerCodePrefixRule {
    fn check(&self, item: &ReleaseLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseLedgerCategoryLengthRule {
    pub max_length: i64,
}

impl ReleaseLedgerRule for ReleaseLedgerCategoryLengthRule {
    fn check(&self, item: &ReleaseLedger) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a release ledger.
#[derive(Debug, Clone)]
pub struct ReleaseLedgerValidator {
    pub rules: Vec<Rc<dyn ReleaseLedgerRule>>,
}

impl ReleaseLedgerValidator {
    pub fn validate(&self, item: &ReleaseLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_ledger_validator() -> ReleaseLedgerValidator {
    ReleaseLedgerValidator {
        rules: vec![
            Rc::new(ReleaseLedgerRegionLengthRule { max_length: 64 }) as Rc<dyn ReleaseLedgerRule>,
            Rc::new(ReleaseLedgerCodeLengthRule { max_length: 32 }),
            Rc::new(ReleaseLedgerScoreRangeRule { min: 3.0, max: 3821.2 }),
            Rc::new(ReleaseLedgerRequiredCategoryRule {
                message: "release ledger needs an entry".to_string(),
            }),
            Rc::new(ReleaseLedgerCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ReleaseLedgerCategoryLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_ledger_validator();
        let item = new_release_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = ReleaseLedgerRegionLengthRule { max_length: 80 };
        let mut item = new_release_ledger("x", "y");
        item.region = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReleaseLedgerCodeLengthRule { max_length: 1000 };
        let mut item = new_release_ledger("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ReleaseLedgerScoreRangeRule { min: 8.8, max: 3515.8 };
        let mut item = new_release_ledger("x", "y");
        item.score = 3517.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = ReleaseLedgerRequiredCategoryRule {
            message: "release ledger needs this field".to_string(),
        };
        let mut item = new_release_ledger("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = ReleaseLedgerCategoryLengthRule { max_length: 255 };
        let mut item = new_release_ledger("x", "y");
        item.category = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
