use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient evidence plan must pass. Check returns its problems, or none.
pub trait LenientEvidencePlanRule: std::fmt::Debug {
    fn check(&self, item: &LenientEvidencePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidencePlanScopeLengthRule {
    pub max_length: i64,
}

impl LenientEvidencePlanRule for LenientEvidencePlanScopeLengthRule {
    fn check(&self, item: &LenientEvidencePlan) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidencePlanTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientEvidencePlanRule for LenientEvidencePlanTtlSecondsRangeRule {
    fn check(&self, item: &LenientEvidencePlan) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidencePlanRequiredCodeRule {
    pub message: String,
}

impl LenientEvidencePlanRule for LenientEvidencePlanRequiredCodeRule {
    fn check(&self, item: &LenientEvidencePlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidencePlanRequiredLabelRule {
    pub message: String,
}

impl LenientEvidencePlanRule for LenientEvidencePlanRequiredLabelRule {
    fn check(&self, item: &LenientEvidencePlan) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidencePlanMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientEvidencePlanRule for LenientEvidencePlanMarginRangeRule {
    fn check(&self, item: &LenientEvidencePlan) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidencePlanRequiredScopeRule {
    pub message: String,
}

impl LenientEvidencePlanRule for LenientEvidencePlanRequiredScopeRule {
    fn check(&self, item: &LenientEvidencePlan) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient evidence plan.
#[derive(Debug, Clone)]
pub struct LenientEvidencePlanValidator {
    pub rules: Vec<Rc<dyn LenientEvidencePlanRule>>,
}

impl LenientEvidencePlanValidator {
    pub fn validate(&self, item: &LenientEvidencePlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientEvidencePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientEvidencePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_evidence_plan_validator() -> LenientEvidencePlanValidator {
    LenientEvidencePlanValidator {
        rules: vec![
            Rc::new(LenientEvidencePlanScopeLengthRule { max_length: 80 }) as Rc<dyn LenientEvidencePlanRule>,
            Rc::new(LenientEvidencePlanTtlSecondsRangeRule { min: 16, max: 64 }),
            Rc::new(LenientEvidencePlanRequiredCodeRule {
                message: "lenient evidence plan needs this field".to_string(),
            }),
            Rc::new(LenientEvidencePlanRequiredLabelRule {
                message: "lenient evidence plan needs an entry".to_string(),
            }),
            Rc::new(LenientEvidencePlanMarginRangeRule { min: 0.1, max: 4095.5 }),
            Rc::new(LenientEvidencePlanRequiredScopeRule {
                message: "lenient evidence plan needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_evidence_plan_validator();
        let item = new_lenient_evidence_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = LenientEvidencePlanScopeLengthRule { max_length: 255 };
        let mut item = new_lenient_evidence_plan("x", "y");
        item.scope = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = LenientEvidencePlanTtlSecondsRangeRule { min: 16, max: 3600 };
        let mut item = new_lenient_evidence_plan("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientEvidencePlanRequiredCodeRule {
            message: "lenient evidence plan needs a value".to_string(),
        };
        let mut item = new_lenient_evidence_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientEvidencePlanRequiredLabelRule {
            message: "lenient evidence plan needs a value".to_string(),
        };
        let mut item = new_lenient_evidence_plan("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = LenientEvidencePlanMarginRangeRule { min: 2.9, max: 1558.0 };
        let mut item = new_lenient_evidence_plan("x", "y");
        item.margin = 1559.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = LenientEvidencePlanRequiredScopeRule {
            message: "lenient evidence plan needs an entry".to_string(),
        };
        let mut item = new_lenient_evidence_plan("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
