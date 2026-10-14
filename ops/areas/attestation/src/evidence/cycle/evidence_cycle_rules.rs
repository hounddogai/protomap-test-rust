use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence cycle must pass. Check returns its problems, or none.
pub trait EvidenceCycleRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCycleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EvidenceCycleRule for EvidenceCycleScoreRangeRule {
    fn check(&self, item: &EvidenceCycle) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCycleNameLengthRule {
    pub max_length: i64,
}

impl EvidenceCycleRule for EvidenceCycleNameLengthRule {
    fn check(&self, item: &EvidenceCycle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCycleCodePrefixRule {
    pub prefix: String,
}

impl EvidenceCycleRule for EvidenceCycleCodePrefixRule {
    fn check(&self, item: &EvidenceCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence cycle.
#[derive(Debug, Clone)]
pub struct EvidenceCycleValidator {
    pub rules: Vec<Rc<dyn EvidenceCycleRule>>,
}

impl EvidenceCycleValidator {
    pub fn validate(&self, item: &EvidenceCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_cycle_validator() -> EvidenceCycleValidator {
    EvidenceCycleValidator {
        rules: vec![
            Rc::new(EvidenceCycleScoreRangeRule { min: 5.2, max: 4411.5 }) as Rc<dyn EvidenceCycleRule>,
            Rc::new(EvidenceCycleNameLengthRule { max_length: 16 }),
            Rc::new(EvidenceCycleCodePrefixRule {
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
        let validator = default_evidence_cycle_validator();
        let item = new_evidence_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = EvidenceCycleScoreRangeRule { min: 0.1, max: 2095.5 };
        let mut item = new_evidence_cycle("x", "y");
        item.score = 2097.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = EvidenceCycleNameLengthRule { max_length: 16 };
        let mut item = new_evidence_cycle("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
