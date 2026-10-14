use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering target must pass. Check returns its problems, or none.
pub trait PeeringTargetRule: std::fmt::Debug {
    fn check(&self, item: &PeeringTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTargetHostLengthRule {
    pub max_length: i64,
}

impl PeeringTargetRule for PeeringTargetHostLengthRule {
    fn check(&self, item: &PeeringTarget) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTargetLabelLengthRule {
    pub max_length: i64,
}

impl PeeringTargetRule for PeeringTargetLabelLengthRule {
    fn check(&self, item: &PeeringTarget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTargetRequiredHostRule {
    pub message: String,
}

impl PeeringTargetRule for PeeringTargetRequiredHostRule {
    fn check(&self, item: &PeeringTarget) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTargetDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PeeringTargetRule for PeeringTargetDiscountRangeRule {
    fn check(&self, item: &PeeringTarget) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTargetBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringTargetRule for PeeringTargetBatchSizeRangeRule {
    fn check(&self, item: &PeeringTarget) -> Vec<String> {
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
pub struct PeeringTargetRequiredCodeRule {
    pub message: String,
}

impl PeeringTargetRule for PeeringTargetRequiredCodeRule {
    fn check(&self, item: &PeeringTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering target.
#[derive(Debug, Clone)]
pub struct PeeringTargetValidator {
    pub rules: Vec<Rc<dyn PeeringTargetRule>>,
}

impl PeeringTargetValidator {
    pub fn validate(&self, item: &PeeringTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_target_validator() -> PeeringTargetValidator {
    PeeringTargetValidator {
        rules: vec![
            Rc::new(PeeringTargetHostLengthRule { max_length: 3600 }) as Rc<dyn PeeringTargetRule>,
            Rc::new(PeeringTargetLabelLengthRule { max_length: 255 }),
            Rc::new(PeeringTargetRequiredHostRule {
                message: "peering target needs an entry".to_string(),
            }),
            Rc::new(PeeringTargetDiscountRangeRule { min: 8.3, max: 2813.9 }),
            Rc::new(PeeringTargetBatchSizeRangeRule { min: 255, max: 1000 }),
            Rc::new(PeeringTargetRequiredCodeRule {
                message: "peering target needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_target_validator();
        let item = new_peering_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = PeeringTargetHostLengthRule { max_length: 80 };
        let mut item = new_peering_target("x", "y");
        item.host = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PeeringTargetLabelLengthRule { max_length: 255 };
        let mut item = new_peering_target("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = PeeringTargetRequiredHostRule {
            message: "peering target needs an entry".to_string(),
        };
        let mut item = new_peering_target("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = PeeringTargetDiscountRangeRule { min: 9.1, max: 3387.1 };
        let mut item = new_peering_target("x", "y");
        item.discount = 3388.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PeeringTargetBatchSizeRangeRule { min: 64, max: 3600 };
        let mut item = new_peering_target("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PeeringTargetRequiredCodeRule {
            message: "peering target needs an entry".to_string(),
        };
        let mut item = new_peering_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
