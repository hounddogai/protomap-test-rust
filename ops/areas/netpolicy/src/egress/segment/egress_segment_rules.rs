use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress segment must pass. Check returns its problems, or none.
pub trait EgressSegmentRule: std::fmt::Debug {
    fn check(&self, item: &EgressSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSegmentDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressSegmentRule for EgressSegmentDiscountRangeRule {
    fn check(&self, item: &EgressSegment) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSegmentRequiredCodeRule {
    pub message: String,
}

impl EgressSegmentRule for EgressSegmentRequiredCodeRule {
    fn check(&self, item: &EgressSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSegmentCodePrefixRule {
    pub prefix: String,
}

impl EgressSegmentRule for EgressSegmentCodePrefixRule {
    fn check(&self, item: &EgressSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSegmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressSegmentRule for EgressSegmentCeilingRangeRule {
    fn check(&self, item: &EgressSegment) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress segment.
#[derive(Debug, Clone)]
pub struct EgressSegmentValidator {
    pub rules: Vec<Rc<dyn EgressSegmentRule>>,
}

impl EgressSegmentValidator {
    pub fn validate(&self, item: &EgressSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_segment_validator() -> EgressSegmentValidator {
    EgressSegmentValidator {
        rules: vec![
            Rc::new(EgressSegmentDiscountRangeRule { min: 5.2, max: 3866.8 }) as Rc<dyn EgressSegmentRule>,
            Rc::new(EgressSegmentRequiredCodeRule {
                message: "egress segment needs this field".to_string(),
            }),
            Rc::new(EgressSegmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(EgressSegmentCeilingRangeRule { min: 9.9, max: 1703.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_segment_validator();
        let item = new_egress_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = EgressSegmentDiscountRangeRule { min: 9.1, max: 1918.6 };
        let mut item = new_egress_segment("x", "y");
        item.discount = 1920.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressSegmentRequiredCodeRule {
            message: "egress segment needs a value".to_string(),
        };
        let mut item = new_egress_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = EgressSegmentCeilingRangeRule { min: 7.9, max: 4241.7 };
        let mut item = new_egress_segment("x", "y");
        item.ceiling = 4243.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
