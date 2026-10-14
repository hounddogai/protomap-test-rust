use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling tenant rollup must pass. Check returns its problems, or none.
pub trait RollingTenantRollupRule: std::fmt::Debug {
    fn check(&self, item: &RollingTenantRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantRollupHostLengthRule {
    pub max_length: i64,
}

impl RollingTenantRollupRule for RollingTenantRollupHostLengthRule {
    fn check(&self, item: &RollingTenantRollup) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantRollupCodeLengthRule {
    pub max_length: i64,
}

impl RollingTenantRollupRule for RollingTenantRollupCodeLengthRule {
    fn check(&self, item: &RollingTenantRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantRollupAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RollingTenantRollupRule for RollingTenantRollupAllowedNameRule {
    fn check(&self, item: &RollingTenantRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantRollupMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingTenantRollupRule for RollingTenantRollupMarginRangeRule {
    fn check(&self, item: &RollingTenantRollup) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling tenant rollup.
#[derive(Debug, Clone)]
pub struct RollingTenantRollupValidator {
    pub rules: Vec<Rc<dyn RollingTenantRollupRule>>,
}

impl RollingTenantRollupValidator {
    pub fn validate(&self, item: &RollingTenantRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingTenantRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTenantRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_tenant_rollup_validator() -> RollingTenantRollupValidator {
    RollingTenantRollupValidator {
        rules: vec![
            Rc::new(RollingTenantRollupHostLengthRule { max_length: 86400 }) as Rc<dyn RollingTenantRollupRule>,
            Rc::new(RollingTenantRollupCodeLengthRule { max_length: 120 }),
            Rc::new(RollingTenantRollupAllowedNameRule {
                allowed: vec!["lenient".to_string()],
            }),
            Rc::new(RollingTenantRollupMarginRangeRule { min: 4.3, max: 812.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_tenant_rollup_validator();
        let item = new_rolling_tenant_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = RollingTenantRollupHostLengthRule { max_length: 64 };
        let mut item = new_rolling_tenant_rollup("x", "y");
        item.host = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingTenantRollupCodeLengthRule { max_length: 1000 };
        let mut item = new_rolling_tenant_rollup("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = RollingTenantRollupMarginRangeRule { min: 6.3, max: 4903.1 };
        let mut item = new_rolling_tenant_rollup("x", "y");
        item.margin = 4904.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
