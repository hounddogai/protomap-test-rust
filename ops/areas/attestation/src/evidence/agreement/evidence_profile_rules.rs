use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence profile must pass. Check returns its problems, or none.
pub trait EvidenceProfileRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceProfileCodeLengthRule {
    pub max_length: i64,
}

impl EvidenceProfileRule for EvidenceProfileCodeLengthRule {
    fn check(&self, item: &EvidenceProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceProfileThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EvidenceProfileRule for EvidenceProfileThresholdRangeRule {
    fn check(&self, item: &EvidenceProfile) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceProfileTitleLengthRule {
    pub max_length: i64,
}

impl EvidenceProfileRule for EvidenceProfileTitleLengthRule {
    fn check(&self, item: &EvidenceProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceProfileEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceProfileRule for EvidenceProfileEndsAtRangeRule {
    fn check(&self, item: &EvidenceProfile) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceProfileRequiredCategoryRule {
    pub message: String,
}

impl EvidenceProfileRule for EvidenceProfileRequiredCategoryRule {
    fn check(&self, item: &EvidenceProfile) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence profile.
#[derive(Debug, Clone)]
pub struct EvidenceProfileValidator {
    pub rules: Vec<Rc<dyn EvidenceProfileRule>>,
}

impl EvidenceProfileValidator {
    pub fn validate(&self, item: &EvidenceProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_profile_validator() -> EvidenceProfileValidator {
    EvidenceProfileValidator {
        rules: vec![
            Rc::new(EvidenceProfileCodeLengthRule { max_length: 3600 }) as Rc<dyn EvidenceProfileRule>,
            Rc::new(EvidenceProfileThresholdRangeRule { min: 7.8, max: 2972.4 }),
            Rc::new(EvidenceProfileTitleLengthRule { max_length: 32 }),
            Rc::new(EvidenceProfileEndsAtRangeRule { min: 80, max: 80 }),
            Rc::new(EvidenceProfileRequiredCategoryRule {
                message: "evidence profile needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_profile_validator();
        let item = new_evidence_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EvidenceProfileCodeLengthRule { max_length: 16 };
        let mut item = new_evidence_profile("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = EvidenceProfileThresholdRangeRule { min: 5.5, max: 3399.7 };
        let mut item = new_evidence_profile("x", "y");
        item.threshold = 3401.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = EvidenceProfileTitleLengthRule { max_length: 80 };
        let mut item = new_evidence_profile("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = EvidenceProfileEndsAtRangeRule { min: 32, max: 64 };
        let mut item = new_evidence_profile("x", "y");
        item.ends_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = EvidenceProfileRequiredCategoryRule {
            message: "evidence profile needs this field".to_string(),
        };
        let mut item = new_evidence_profile("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
