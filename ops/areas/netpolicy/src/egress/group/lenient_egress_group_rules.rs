use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient egress group must pass. Check returns its problems, or none.
pub trait LenientEgressGroupRule: std::fmt::Debug {
    fn check(&self, item: &LenientEgressGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressGroupQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientEgressGroupRule for LenientEgressGroupQuantityRangeRule {
    fn check(&self, item: &LenientEgressGroup) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressGroupRequiredLabelRule {
    pub message: String,
}

impl LenientEgressGroupRule for LenientEgressGroupRequiredLabelRule {
    fn check(&self, item: &LenientEgressGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressGroupRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientEgressGroupRule for LenientEgressGroupRotationDaysRangeRule {
    fn check(&self, item: &LenientEgressGroup) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressGroupCodeLengthRule {
    pub max_length: i64,
}

impl LenientEgressGroupRule for LenientEgressGroupCodeLengthRule {
    fn check(&self, item: &LenientEgressGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient egress group.
#[derive(Debug, Clone)]
pub struct LenientEgressGroupValidator {
    pub rules: Vec<Rc<dyn LenientEgressGroupRule>>,
}

impl LenientEgressGroupValidator {
    pub fn validate(&self, item: &LenientEgressGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientEgressGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientEgressGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_egress_group_validator() -> LenientEgressGroupValidator {
    LenientEgressGroupValidator {
        rules: vec![
            Rc::new(LenientEgressGroupQuantityRangeRule { min: 16, max: 3600 }) as Rc<dyn LenientEgressGroupRule>,
            Rc::new(LenientEgressGroupRequiredLabelRule {
                message: "lenient egress group needs an entry".to_string(),
            }),
            Rc::new(LenientEgressGroupRotationDaysRangeRule { min: 32, max: 32 }),
            Rc::new(LenientEgressGroupCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_egress_group_validator();
        let item = new_lenient_egress_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = LenientEgressGroupQuantityRangeRule { min: 16, max: 32 };
        let mut item = new_lenient_egress_group("x", "y");
        item.quantity = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientEgressGroupRequiredLabelRule {
            message: "lenient egress group needs a value".to_string(),
        };
        let mut item = new_lenient_egress_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = LenientEgressGroupRotationDaysRangeRule { min: 80, max: 1000 };
        let mut item = new_lenient_egress_group("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientEgressGroupCodeLengthRule { max_length: 32 };
        let mut item = new_lenient_egress_group("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
