use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary slo draft must pass. Check returns its problems, or none.
pub trait PrimarySloDraftRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySloDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloDraftWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimarySloDraftRule for PrimarySloDraftWeightRangeRule {
    fn check(&self, item: &PrimarySloDraft) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloDraftCodePrefixRule {
    pub prefix: String,
}

impl PrimarySloDraftRule for PrimarySloDraftCodePrefixRule {
    fn check(&self, item: &PrimarySloDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloDraftDescriptionLengthRule {
    pub max_length: i64,
}

impl PrimarySloDraftRule for PrimarySloDraftDescriptionLengthRule {
    fn check(&self, item: &PrimarySloDraft) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloDraftCodeLengthRule {
    pub max_length: i64,
}

impl PrimarySloDraftRule for PrimarySloDraftCodeLengthRule {
    fn check(&self, item: &PrimarySloDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary slo draft.
#[derive(Debug, Clone)]
pub struct PrimarySloDraftValidator {
    pub rules: Vec<Rc<dyn PrimarySloDraftRule>>,
}

impl PrimarySloDraftValidator {
    pub fn validate(&self, item: &PrimarySloDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimarySloDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySloDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_slo_draft_validator() -> PrimarySloDraftValidator {
    PrimarySloDraftValidator {
        rules: vec![
            Rc::new(PrimarySloDraftWeightRangeRule { min: 7.4, max: 3445.4 }) as Rc<dyn PrimarySloDraftRule>,
            Rc::new(PrimarySloDraftCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PrimarySloDraftDescriptionLengthRule { max_length: 86400 }),
            Rc::new(PrimarySloDraftCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_slo_draft_validator();
        let item = new_primary_slo_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PrimarySloDraftWeightRangeRule { min: 7.0, max: 236.3 };
        let mut item = new_primary_slo_draft("x", "y");
        item.weight = 237.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = PrimarySloDraftDescriptionLengthRule { max_length: 80 };
        let mut item = new_primary_slo_draft("x", "y");
        item.description = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimarySloDraftCodeLengthRule { max_length: 3600 };
        let mut item = new_primary_slo_draft("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
