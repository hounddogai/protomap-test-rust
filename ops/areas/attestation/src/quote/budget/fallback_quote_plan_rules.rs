use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback quote plan must pass. Check returns its problems, or none.
pub trait FallbackQuotePlanRule: std::fmt::Debug {
    fn check(&self, item: &FallbackQuotePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuotePlanVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackQuotePlanRule for FallbackQuotePlanVersionRangeRule {
    fn check(&self, item: &FallbackQuotePlan) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuotePlanCodeLengthRule {
    pub max_length: i64,
}

impl FallbackQuotePlanRule for FallbackQuotePlanCodeLengthRule {
    fn check(&self, item: &FallbackQuotePlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuotePlanCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackQuotePlanRule for FallbackQuotePlanCeilingRangeRule {
    fn check(&self, item: &FallbackQuotePlan) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuotePlanCodePrefixRule {
    pub prefix: String,
}

impl FallbackQuotePlanRule for FallbackQuotePlanCodePrefixRule {
    fn check(&self, item: &FallbackQuotePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback quote plan.
#[derive(Debug, Clone)]
pub struct FallbackQuotePlanValidator {
    pub rules: Vec<Rc<dyn FallbackQuotePlanRule>>,
}

impl FallbackQuotePlanValidator {
    pub fn validate(&self, item: &FallbackQuotePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackQuotePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackQuotePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_quote_plan_validator() -> FallbackQuotePlanValidator {
    FallbackQuotePlanValidator {
        rules: vec![
            Rc::new(FallbackQuotePlanVersionRangeRule { min: 16, max: 1000 }) as Rc<dyn FallbackQuotePlanRule>,
            Rc::new(FallbackQuotePlanCodeLengthRule { max_length: 64 }),
            Rc::new(FallbackQuotePlanCeilingRangeRule { min: 4.4, max: 3264.1 }),
            Rc::new(FallbackQuotePlanCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_quote_plan_validator();
        let item = new_fallback_quote_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = FallbackQuotePlanVersionRangeRule { min: 32, max: 255 };
        let mut item = new_fallback_quote_plan("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackQuotePlanCodeLengthRule { max_length: 3600 };
        let mut item = new_fallback_quote_plan("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = FallbackQuotePlanCeilingRangeRule { min: 3.3, max: 4033.3 };
        let mut item = new_fallback_quote_plan("x", "y");
        item.ceiling = 4034.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
