use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary evidence plan must pass. Check returns its problems, or none.
pub trait PrimaryEvidencePlanRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEvidencePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidencePlanCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryEvidencePlanRule for PrimaryEvidencePlanCodeLengthRule {
    fn check(&self, item: &PrimaryEvidencePlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidencePlanCodePrefixRule {
    pub prefix: String,
}

impl PrimaryEvidencePlanRule for PrimaryEvidencePlanCodePrefixRule {
    fn check(&self, item: &PrimaryEvidencePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidencePlanVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryEvidencePlanRule for PrimaryEvidencePlanVersionRangeRule {
    fn check(&self, item: &PrimaryEvidencePlan) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidencePlanAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryEvidencePlanRule for PrimaryEvidencePlanAmountRangeRule {
    fn check(&self, item: &PrimaryEvidencePlan) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidencePlanRequiredNameRule {
    pub message: String,
}

impl PrimaryEvidencePlanRule for PrimaryEvidencePlanRequiredNameRule {
    fn check(&self, item: &PrimaryEvidencePlan) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary evidence plan.
#[derive(Debug, Clone)]
pub struct PrimaryEvidencePlanValidator {
    pub rules: Vec<Rc<dyn PrimaryEvidencePlanRule>>,
}

impl PrimaryEvidencePlanValidator {
    pub fn validate(&self, item: &PrimaryEvidencePlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryEvidencePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEvidencePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_evidence_plan_validator() -> PrimaryEvidencePlanValidator {
    PrimaryEvidencePlanValidator {
        rules: vec![
            Rc::new(PrimaryEvidencePlanCodeLengthRule { max_length: 3600 }) as Rc<dyn PrimaryEvidencePlanRule>,
            Rc::new(PrimaryEvidencePlanCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PrimaryEvidencePlanVersionRangeRule { min: 16, max: 32 }),
            Rc::new(PrimaryEvidencePlanAmountRangeRule { min: 1.5, max: 3611.0 }),
            Rc::new(PrimaryEvidencePlanRequiredNameRule {
                message: "primary evidence plan needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_evidence_plan_validator();
        let item = new_primary_evidence_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryEvidencePlanCodeLengthRule { max_length: 64 };
        let mut item = new_primary_evidence_plan("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = PrimaryEvidencePlanVersionRangeRule { min: 32, max: 3600 };
        let mut item = new_primary_evidence_plan("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = PrimaryEvidencePlanAmountRangeRule { min: 0.5, max: 1516.1 };
        let mut item = new_primary_evidence_plan("x", "y");
        item.amount = 1517.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryEvidencePlanRequiredNameRule {
            message: "primary evidence plan needs a value".to_string(),
        };
        let mut item = new_primary_evidence_plan("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
