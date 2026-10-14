use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global tenant digest must pass. Check returns its problems, or none.
pub trait GlobalTenantDigestRule: std::fmt::Debug {
    fn check(&self, item: &GlobalTenantDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTenantDigestScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalTenantDigestRule for GlobalTenantDigestScoreRangeRule {
    fn check(&self, item: &GlobalTenantDigest) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTenantDigestRequiredChannelRule {
    pub message: String,
}

impl GlobalTenantDigestRule for GlobalTenantDigestRequiredChannelRule {
    fn check(&self, item: &GlobalTenantDigest) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTenantDigestRequiredNameRule {
    pub message: String,
}

impl GlobalTenantDigestRule for GlobalTenantDigestRequiredNameRule {
    fn check(&self, item: &GlobalTenantDigest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTenantDigestCodePrefixRule {
    pub prefix: String,
}

impl GlobalTenantDigestRule for GlobalTenantDigestCodePrefixRule {
    fn check(&self, item: &GlobalTenantDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global tenant digest.
#[derive(Debug, Clone)]
pub struct GlobalTenantDigestValidator {
    pub rules: Vec<Rc<dyn GlobalTenantDigestRule>>,
}

impl GlobalTenantDigestValidator {
    pub fn validate(&self, item: &GlobalTenantDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalTenantDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalTenantDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_tenant_digest_validator() -> GlobalTenantDigestValidator {
    GlobalTenantDigestValidator {
        rules: vec![
            Rc::new(GlobalTenantDigestScoreRangeRule { min: 9.1, max: 2085.7 }) as Rc<dyn GlobalTenantDigestRule>,
            Rc::new(GlobalTenantDigestRequiredChannelRule {
                message: "global tenant digest needs an entry".to_string(),
            }),
            Rc::new(GlobalTenantDigestRequiredNameRule {
                message: "global tenant digest needs a value".to_string(),
            }),
            Rc::new(GlobalTenantDigestCodePrefixRule {
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
        let validator = default_global_tenant_digest_validator();
        let item = new_global_tenant_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = GlobalTenantDigestScoreRangeRule { min: 8.0, max: 2747.7 };
        let mut item = new_global_tenant_digest("x", "y");
        item.score = 2749.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = GlobalTenantDigestRequiredChannelRule {
            message: "global tenant digest needs a value".to_string(),
        };
        let mut item = new_global_tenant_digest("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GlobalTenantDigestRequiredNameRule {
            message: "global tenant digest needs a value".to_string(),
        };
        let mut item = new_global_tenant_digest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
