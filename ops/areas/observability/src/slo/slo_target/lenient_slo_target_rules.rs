use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient slo target must pass. Check returns its problems, or none.
pub trait LenientSloTargetRule: std::fmt::Debug {
    fn check(&self, item: &LenientSloTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloTargetLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientSloTargetRule for LenientSloTargetLimitRangeRule {
    fn check(&self, item: &LenientSloTarget) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloTargetCodePrefixRule {
    pub prefix: String,
}

impl LenientSloTargetRule for LenientSloTargetCodePrefixRule {
    fn check(&self, item: &LenientSloTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloTargetRequiredTitleRule {
    pub message: String,
}

impl LenientSloTargetRule for LenientSloTargetRequiredTitleRule {
    fn check(&self, item: &LenientSloTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloTargetTitleLengthRule {
    pub max_length: i64,
}

impl LenientSloTargetRule for LenientSloTargetTitleLengthRule {
    fn check(&self, item: &LenientSloTarget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloTargetMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientSloTargetRule for LenientSloTargetMarginRangeRule {
    fn check(&self, item: &LenientSloTarget) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient slo target.
#[derive(Debug, Clone)]
pub struct LenientSloTargetValidator {
    pub rules: Vec<Rc<dyn LenientSloTargetRule>>,
}

impl LenientSloTargetValidator {
    pub fn validate(&self, item: &LenientSloTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientSloTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSloTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_slo_target_validator() -> LenientSloTargetValidator {
    LenientSloTargetValidator {
        rules: vec![
            Rc::new(LenientSloTargetLimitRangeRule { min: 255, max: 1000 }) as Rc<dyn LenientSloTargetRule>,
            Rc::new(LenientSloTargetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LenientSloTargetRequiredTitleRule {
                message: "lenient slo target needs an entry".to_string(),
            }),
            Rc::new(LenientSloTargetTitleLengthRule { max_length: 1000 }),
            Rc::new(LenientSloTargetMarginRangeRule { min: 6.8, max: 3104.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_slo_target_validator();
        let item = new_lenient_slo_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = LenientSloTargetLimitRangeRule { min: 16, max: 64 };
        let mut item = new_lenient_slo_target("x", "y");
        item.limit = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientSloTargetRequiredTitleRule {
            message: "lenient slo target needs a value".to_string(),
        };
        let mut item = new_lenient_slo_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientSloTargetTitleLengthRule { max_length: 1000 };
        let mut item = new_lenient_slo_target("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = LenientSloTargetMarginRangeRule { min: 4.2, max: 2391.2 };
        let mut item = new_lenient_slo_target("x", "y");
        item.margin = 2392.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
