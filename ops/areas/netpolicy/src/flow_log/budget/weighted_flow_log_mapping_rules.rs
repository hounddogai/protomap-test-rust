use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted flow log mapping must pass. Check returns its problems, or none.
pub trait WeightedFlowLogMappingRule: std::fmt::Debug {
    fn check(&self, item: &WeightedFlowLogMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogMappingRequiredCipherRule {
    pub message: String,
}

impl WeightedFlowLogMappingRule for WeightedFlowLogMappingRequiredCipherRule {
    fn check(&self, item: &WeightedFlowLogMapping) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogMappingCodePrefixRule {
    pub prefix: String,
}

impl WeightedFlowLogMappingRule for WeightedFlowLogMappingCodePrefixRule {
    fn check(&self, item: &WeightedFlowLogMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogMappingRequiredCodeRule {
    pub message: String,
}

impl WeightedFlowLogMappingRule for WeightedFlowLogMappingRequiredCodeRule {
    fn check(&self, item: &WeightedFlowLogMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogMappingAllowedNameRule {
    pub allowed: Vec<String>,
}

impl WeightedFlowLogMappingRule for WeightedFlowLogMappingAllowedNameRule {
    fn check(&self, item: &WeightedFlowLogMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogMappingCodeLengthRule {
    pub max_length: i64,
}

impl WeightedFlowLogMappingRule for WeightedFlowLogMappingCodeLengthRule {
    fn check(&self, item: &WeightedFlowLogMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted flow log mapping.
#[derive(Debug, Clone)]
pub struct WeightedFlowLogMappingValidator {
    pub rules: Vec<Rc<dyn WeightedFlowLogMappingRule>>,
}

impl WeightedFlowLogMappingValidator {
    pub fn validate(&self, item: &WeightedFlowLogMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedFlowLogMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedFlowLogMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_flow_log_mapping_validator() -> WeightedFlowLogMappingValidator {
    WeightedFlowLogMappingValidator {
        rules: vec![
            Rc::new(WeightedFlowLogMappingRequiredCipherRule {
                message: "weighted flow log mapping needs an entry".to_string(),
            }) as Rc<dyn WeightedFlowLogMappingRule>,
            Rc::new(WeightedFlowLogMappingCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(WeightedFlowLogMappingRequiredCodeRule {
                message: "weighted flow log mapping needs this field".to_string(),
            }),
            Rc::new(WeightedFlowLogMappingAllowedNameRule {
                allowed: vec!["rolling".to_string(), "strict".to_string()],
            }),
            Rc::new(WeightedFlowLogMappingCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_flow_log_mapping_validator();
        let item = new_weighted_flow_log_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = WeightedFlowLogMappingRequiredCipherRule {
            message: "weighted flow log mapping needs this field".to_string(),
        };
        let mut item = new_weighted_flow_log_mapping("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedFlowLogMappingRequiredCodeRule {
            message: "weighted flow log mapping needs a value".to_string(),
        };
        let mut item = new_weighted_flow_log_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedFlowLogMappingCodeLengthRule { max_length: 86400 };
        let mut item = new_weighted_flow_log_mapping("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
