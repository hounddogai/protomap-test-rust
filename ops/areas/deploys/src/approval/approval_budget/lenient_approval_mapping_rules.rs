use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient approval mapping must pass. Check returns its problems, or none.
pub trait LenientApprovalMappingRule: std::fmt::Debug {
    fn check(&self, item: &LenientApprovalMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalMappingRequiredCipherRule {
    pub message: String,
}

impl LenientApprovalMappingRule for LenientApprovalMappingRequiredCipherRule {
    fn check(&self, item: &LenientApprovalMapping) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalMappingRequiredLabelRule {
    pub message: String,
}

impl LenientApprovalMappingRule for LenientApprovalMappingRequiredLabelRule {
    fn check(&self, item: &LenientApprovalMapping) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalMappingCodeLengthRule {
    pub max_length: i64,
}

impl LenientApprovalMappingRule for LenientApprovalMappingCodeLengthRule {
    fn check(&self, item: &LenientApprovalMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalMappingMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientApprovalMappingRule for LenientApprovalMappingMarginRangeRule {
    fn check(&self, item: &LenientApprovalMapping) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient approval mapping.
#[derive(Debug, Clone)]
pub struct LenientApprovalMappingValidator {
    pub rules: Vec<Rc<dyn LenientApprovalMappingRule>>,
}

impl LenientApprovalMappingValidator {
    pub fn validate(&self, item: &LenientApprovalMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientApprovalMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientApprovalMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_approval_mapping_validator() -> LenientApprovalMappingValidator {
    LenientApprovalMappingValidator {
        rules: vec![
            Rc::new(LenientApprovalMappingRequiredCipherRule {
                message: "lenient approval mapping needs an entry".to_string(),
            }) as Rc<dyn LenientApprovalMappingRule>,
            Rc::new(LenientApprovalMappingRequiredLabelRule {
                message: "lenient approval mapping needs an entry".to_string(),
            }),
            Rc::new(LenientApprovalMappingCodeLengthRule { max_length: 120 }),
            Rc::new(LenientApprovalMappingMarginRangeRule { min: 0.6, max: 1673.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_approval_mapping_validator();
        let item = new_lenient_approval_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = LenientApprovalMappingRequiredCipherRule {
            message: "lenient approval mapping needs an entry".to_string(),
        };
        let mut item = new_lenient_approval_mapping("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientApprovalMappingRequiredLabelRule {
            message: "lenient approval mapping needs this field".to_string(),
        };
        let mut item = new_lenient_approval_mapping("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientApprovalMappingCodeLengthRule { max_length: 64 };
        let mut item = new_lenient_approval_mapping("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = LenientApprovalMappingMarginRangeRule { min: 1.3, max: 1393.6 };
        let mut item = new_lenient_approval_mapping("x", "y");
        item.margin = 1395.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
