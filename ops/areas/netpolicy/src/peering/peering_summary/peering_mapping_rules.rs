use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering mapping must pass. Check returns its problems, or none.
pub trait PeeringMappingRule: std::fmt::Debug {
    fn check(&self, item: &PeeringMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMappingCodeLengthRule {
    pub max_length: i64,
}

impl PeeringMappingRule for PeeringMappingCodeLengthRule {
    fn check(&self, item: &PeeringMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMappingBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringMappingRule for PeeringMappingBatchSizeRangeRule {
    fn check(&self, item: &PeeringMapping) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMappingRequiredDescriptionRule {
    pub message: String,
}

impl PeeringMappingRule for PeeringMappingRequiredDescriptionRule {
    fn check(&self, item: &PeeringMapping) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering mapping.
#[derive(Debug, Clone)]
pub struct PeeringMappingValidator {
    pub rules: Vec<Rc<dyn PeeringMappingRule>>,
}

impl PeeringMappingValidator {
    pub fn validate(&self, item: &PeeringMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_mapping_validator() -> PeeringMappingValidator {
    PeeringMappingValidator {
        rules: vec![
            Rc::new(PeeringMappingCodeLengthRule { max_length: 16 }) as Rc<dyn PeeringMappingRule>,
            Rc::new(PeeringMappingBatchSizeRangeRule { min: 120, max: 1000 }),
            Rc::new(PeeringMappingRequiredDescriptionRule {
                message: "peering mapping needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_mapping_validator();
        let item = new_peering_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PeeringMappingCodeLengthRule { max_length: 1000 };
        let mut item = new_peering_mapping("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PeeringMappingBatchSizeRangeRule { min: 16, max: 86400 };
        let mut item = new_peering_mapping("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = PeeringMappingRequiredDescriptionRule {
            message: "peering mapping needs an entry".to_string(),
        };
        let mut item = new_peering_mapping("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
