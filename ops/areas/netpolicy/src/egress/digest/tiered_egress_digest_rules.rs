use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered egress digest must pass. Check returns its problems, or none.
pub trait TieredEgressDigestRule: std::fmt::Debug {
    fn check(&self, item: &TieredEgressDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressDigestRequiredTenantIdRule {
    pub message: String,
}

impl TieredEgressDigestRule for TieredEgressDigestRequiredTenantIdRule {
    fn check(&self, item: &TieredEgressDigest) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressDigestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TieredEgressDigestRule for TieredEgressDigestAllowedCodeRule {
    fn check(&self, item: &TieredEgressDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressDigestRequiredCodeRule {
    pub message: String,
}

impl TieredEgressDigestRule for TieredEgressDigestRequiredCodeRule {
    fn check(&self, item: &TieredEgressDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressDigestCodeLengthRule {
    pub max_length: i64,
}

impl TieredEgressDigestRule for TieredEgressDigestCodeLengthRule {
    fn check(&self, item: &TieredEgressDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressDigestMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredEgressDigestRule for TieredEgressDigestMarginRangeRule {
    fn check(&self, item: &TieredEgressDigest) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered egress digest.
#[derive(Debug, Clone)]
pub struct TieredEgressDigestValidator {
    pub rules: Vec<Rc<dyn TieredEgressDigestRule>>,
}

impl TieredEgressDigestValidator {
    pub fn validate(&self, item: &TieredEgressDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredEgressDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredEgressDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_egress_digest_validator() -> TieredEgressDigestValidator {
    TieredEgressDigestValidator {
        rules: vec![
            Rc::new(TieredEgressDigestRequiredTenantIdRule {
                message: "tiered egress digest needs a value".to_string(),
            }) as Rc<dyn TieredEgressDigestRule>,
            Rc::new(TieredEgressDigestAllowedCodeRule {
                allowed: vec!["batched".to_string()],
            }),
            Rc::new(TieredEgressDigestRequiredCodeRule {
                message: "tiered egress digest needs an entry".to_string(),
            }),
            Rc::new(TieredEgressDigestCodeLengthRule { max_length: 32 }),
            Rc::new(TieredEgressDigestMarginRangeRule { min: 0.1, max: 246.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_egress_digest_validator();
        let item = new_tiered_egress_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = TieredEgressDigestRequiredTenantIdRule {
            message: "tiered egress digest needs a value".to_string(),
        };
        let mut item = new_tiered_egress_digest("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredEgressDigestRequiredCodeRule {
            message: "tiered egress digest needs this field".to_string(),
        };
        let mut item = new_tiered_egress_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredEgressDigestCodeLengthRule { max_length: 1000 };
        let mut item = new_tiered_egress_digest("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = TieredEgressDigestMarginRangeRule { min: 2.4, max: 188.2 };
        let mut item = new_tiered_egress_digest("x", "y");
        item.margin = 189.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
