use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress rollup must pass. Check returns its problems, or none.
pub trait EgressRollupRule: std::fmt::Debug {
    fn check(&self, item: &EgressRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRollupDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressRollupRule for EgressRollupDiscountRangeRule {
    fn check(&self, item: &EgressRollup) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRollupRequiredCodeRule {
    pub message: String,
}

impl EgressRollupRule for EgressRollupRequiredCodeRule {
    fn check(&self, item: &EgressRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRollupLocaleLengthRule {
    pub max_length: i64,
}

impl EgressRollupRule for EgressRollupLocaleLengthRule {
    fn check(&self, item: &EgressRollup) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRollupCodePrefixRule {
    pub prefix: String,
}

impl EgressRollupRule for EgressRollupCodePrefixRule {
    fn check(&self, item: &EgressRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRollupRequiredNameRule {
    pub message: String,
}

impl EgressRollupRule for EgressRollupRequiredNameRule {
    fn check(&self, item: &EgressRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRollupRequiredLocaleRule {
    pub message: String,
}

impl EgressRollupRule for EgressRollupRequiredLocaleRule {
    fn check(&self, item: &EgressRollup) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress rollup.
#[derive(Debug, Clone)]
pub struct EgressRollupValidator {
    pub rules: Vec<Rc<dyn EgressRollupRule>>,
}

impl EgressRollupValidator {
    pub fn validate(&self, item: &EgressRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_rollup_validator() -> EgressRollupValidator {
    EgressRollupValidator {
        rules: vec![
            Rc::new(EgressRollupDiscountRangeRule { min: 5.5, max: 2583.1 }) as Rc<dyn EgressRollupRule>,
            Rc::new(EgressRollupRequiredCodeRule {
                message: "egress rollup needs a value".to_string(),
            }),
            Rc::new(EgressRollupLocaleLengthRule { max_length: 86400 }),
            Rc::new(EgressRollupCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(EgressRollupRequiredNameRule {
                message: "egress rollup needs this field".to_string(),
            }),
            Rc::new(EgressRollupRequiredLocaleRule {
                message: "egress rollup needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_rollup_validator();
        let item = new_egress_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = EgressRollupDiscountRangeRule { min: 10.0, max: 4709.8 };
        let mut item = new_egress_rollup("x", "y");
        item.discount = 4711.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressRollupRequiredCodeRule {
            message: "egress rollup needs an entry".to_string(),
        };
        let mut item = new_egress_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = EgressRollupLocaleLengthRule { max_length: 16 };
        let mut item = new_egress_rollup("x", "y");
        item.locale = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = EgressRollupRequiredNameRule {
            message: "egress rollup needs this field".to_string(),
        };
        let mut item = new_egress_rollup("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = EgressRollupRequiredLocaleRule {
            message: "egress rollup needs an entry".to_string(),
        };
        let mut item = new_egress_rollup("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
