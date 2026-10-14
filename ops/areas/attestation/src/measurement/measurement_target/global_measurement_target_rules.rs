use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global measurement target must pass. Check returns its problems, or none.
pub trait GlobalMeasurementTargetRule: std::fmt::Debug {
    fn check(&self, item: &GlobalMeasurementTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementTargetAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl GlobalMeasurementTargetRule for GlobalMeasurementTargetAllowedIssuerRule {
    fn check(&self, item: &GlobalMeasurementTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is unknown", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementTargetCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalMeasurementTargetRule for GlobalMeasurementTargetCeilingRangeRule {
    fn check(&self, item: &GlobalMeasurementTarget) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementTargetIssuerLengthRule {
    pub max_length: i64,
}

impl GlobalMeasurementTargetRule for GlobalMeasurementTargetIssuerLengthRule {
    fn check(&self, item: &GlobalMeasurementTarget) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementTargetLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalMeasurementTargetRule for GlobalMeasurementTargetLimitRangeRule {
    fn check(&self, item: &GlobalMeasurementTarget) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementTargetCodePrefixRule {
    pub prefix: String,
}

impl GlobalMeasurementTargetRule for GlobalMeasurementTargetCodePrefixRule {
    fn check(&self, item: &GlobalMeasurementTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global measurement target.
#[derive(Debug, Clone)]
pub struct GlobalMeasurementTargetValidator {
    pub rules: Vec<Rc<dyn GlobalMeasurementTargetRule>>,
}

impl GlobalMeasurementTargetValidator {
    pub fn validate(&self, item: &GlobalMeasurementTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalMeasurementTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalMeasurementTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_measurement_target_validator() -> GlobalMeasurementTargetValidator {
    GlobalMeasurementTargetValidator {
        rules: vec![
            Rc::new(GlobalMeasurementTargetAllowedIssuerRule {
                allowed: vec!["pending".to_string(), "pending".to_string()],
            }) as Rc<dyn GlobalMeasurementTargetRule>,
            Rc::new(GlobalMeasurementTargetCeilingRangeRule { min: 6.9, max: 2944.7 }),
            Rc::new(GlobalMeasurementTargetIssuerLengthRule { max_length: 1000 }),
            Rc::new(GlobalMeasurementTargetLimitRangeRule { min: 80, max: 86400 }),
            Rc::new(GlobalMeasurementTargetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_measurement_target_validator();
        let item = new_global_measurement_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = GlobalMeasurementTargetCeilingRangeRule { min: 0.4, max: 3074.1 };
        let mut item = new_global_measurement_target("x", "y");
        item.ceiling = 3075.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = GlobalMeasurementTargetIssuerLengthRule { max_length: 1000 };
        let mut item = new_global_measurement_target("x", "y");
        item.issuer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = GlobalMeasurementTargetLimitRangeRule { min: 16, max: 255 };
        let mut item = new_global_measurement_target("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
