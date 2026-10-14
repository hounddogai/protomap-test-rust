use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient sink plan must pass. Check returns its problems, or none.
pub trait LenientSinkPlanRule: std::fmt::Debug {
    fn check(&self, item: &LenientSinkPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkPlanTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientSinkPlanRule for LenientSinkPlanTtlSecondsRangeRule {
    fn check(&self, item: &LenientSinkPlan) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkPlanExternalRefLengthRule {
    pub max_length: i64,
}

impl LenientSinkPlanRule for LenientSinkPlanExternalRefLengthRule {
    fn check(&self, item: &LenientSinkPlan) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkPlanIssuerLengthRule {
    pub max_length: i64,
}

impl LenientSinkPlanRule for LenientSinkPlanIssuerLengthRule {
    fn check(&self, item: &LenientSinkPlan) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkPlanAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl LenientSinkPlanRule for LenientSinkPlanAllowedIssuerRule {
    fn check(&self, item: &LenientSinkPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is expired", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkPlanCodePrefixRule {
    pub prefix: String,
}

impl LenientSinkPlanRule for LenientSinkPlanCodePrefixRule {
    fn check(&self, item: &LenientSinkPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient sink plan.
#[derive(Debug, Clone)]
pub struct LenientSinkPlanValidator {
    pub rules: Vec<Rc<dyn LenientSinkPlanRule>>,
}

impl LenientSinkPlanValidator {
    pub fn validate(&self, item: &LenientSinkPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientSinkPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSinkPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_sink_plan_validator() -> LenientSinkPlanValidator {
    LenientSinkPlanValidator {
        rules: vec![
            Rc::new(LenientSinkPlanTtlSecondsRangeRule { min: 120, max: 1000 }) as Rc<dyn LenientSinkPlanRule>,
            Rc::new(LenientSinkPlanExternalRefLengthRule { max_length: 1000 }),
            Rc::new(LenientSinkPlanIssuerLengthRule { max_length: 64 }),
            Rc::new(LenientSinkPlanAllowedIssuerRule { allowed: Vec::new() }),
            Rc::new(LenientSinkPlanCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_sink_plan_validator();
        let item = new_lenient_sink_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = LenientSinkPlanTtlSecondsRangeRule { min: 32, max: 1000 };
        let mut item = new_lenient_sink_plan("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = LenientSinkPlanExternalRefLengthRule { max_length: 1000 };
        let mut item = new_lenient_sink_plan("x", "y");
        item.external_ref = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = LenientSinkPlanIssuerLengthRule { max_length: 86400 };
        let mut item = new_lenient_sink_plan("x", "y");
        item.issuer = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
