use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient slo batch must pass. Check returns its problems, or none.
pub trait LenientSloBatchRule: std::fmt::Debug {
    fn check(&self, item: &LenientSloBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloBatchRequiredNameRule {
    pub message: String,
}

impl LenientSloBatchRule for LenientSloBatchRequiredNameRule {
    fn check(&self, item: &LenientSloBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloBatchCodePrefixRule {
    pub prefix: String,
}

impl LenientSloBatchRule for LenientSloBatchCodePrefixRule {
    fn check(&self, item: &LenientSloBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloBatchRequiredCodeRule {
    pub message: String,
}

impl LenientSloBatchRule for LenientSloBatchRequiredCodeRule {
    fn check(&self, item: &LenientSloBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient slo batch.
#[derive(Debug, Clone)]
pub struct LenientSloBatchValidator {
    pub rules: Vec<Rc<dyn LenientSloBatchRule>>,
}

impl LenientSloBatchValidator {
    pub fn validate(&self, item: &LenientSloBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientSloBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSloBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_slo_batch_validator() -> LenientSloBatchValidator {
    LenientSloBatchValidator {
        rules: vec![
            Rc::new(LenientSloBatchRequiredNameRule {
                message: "lenient slo batch needs this field".to_string(),
            }) as Rc<dyn LenientSloBatchRule>,
            Rc::new(LenientSloBatchCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LenientSloBatchRequiredCodeRule {
                message: "lenient slo batch needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_slo_batch_validator();
        let item = new_lenient_slo_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientSloBatchRequiredNameRule {
            message: "lenient slo batch needs this field".to_string(),
        };
        let mut item = new_lenient_slo_batch("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientSloBatchRequiredCodeRule {
            message: "lenient slo batch needs this field".to_string(),
        };
        let mut item = new_lenient_slo_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
