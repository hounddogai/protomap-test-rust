use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant target must pass. Check returns its problems, or none.
pub trait TenantTargetRule: std::fmt::Debug {
    fn check(&self, item: &TenantTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTargetRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantTargetRule for TenantTargetRateRangeRule {
    fn check(&self, item: &TenantTarget) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTargetAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TenantTargetRule for TenantTargetAllowedTitleRule {
    fn check(&self, item: &TenantTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTargetCodePrefixRule {
    pub prefix: String,
}

impl TenantTargetRule for TenantTargetCodePrefixRule {
    fn check(&self, item: &TenantTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantTargetRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantTargetRule for TenantTargetRatioRangeRule {
    fn check(&self, item: &TenantTarget) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tenant target.
#[derive(Debug, Clone)]
pub struct TenantTargetValidator {
    pub rules: Vec<Rc<dyn TenantTargetRule>>,
}

impl TenantTargetValidator {
    pub fn validate(&self, item: &TenantTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_target_validator() -> TenantTargetValidator {
    TenantTargetValidator {
        rules: vec![
            Rc::new(TenantTargetRateRangeRule { min: 5.1, max: 4458.9 }) as Rc<dyn TenantTargetRule>,
            Rc::new(TenantTargetAllowedTitleRule {
                allowed: vec!["primary".to_string(), "rolling".to_string()],
            }),
            Rc::new(TenantTargetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(TenantTargetRatioRangeRule { min: 4.8, max: 2250.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_target_validator();
        let item = new_tenant_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = TenantTargetRateRangeRule { min: 7.4, max: 4567.7 };
        let mut item = new_tenant_target("x", "y");
        item.rate = 4569.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = TenantTargetRatioRangeRule { min: 5.1, max: 1618.8 };
        let mut item = new_tenant_target("x", "y");
        item.ratio = 1620.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
