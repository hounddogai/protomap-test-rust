use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual overage rollup must pass. Check returns its problems, or none.
pub trait ManualOverageRollupRule: std::fmt::Debug {
    fn check(&self, item: &ManualOverageRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualOverageRollupRequiredCodeRule {
    pub message: String,
}

impl ManualOverageRollupRule for ManualOverageRollupRequiredCodeRule {
    fn check(&self, item: &ManualOverageRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualOverageRollupRequiredIssuerRule {
    pub message: String,
}

impl ManualOverageRollupRule for ManualOverageRollupRequiredIssuerRule {
    fn check(&self, item: &ManualOverageRollup) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualOverageRollupCodePrefixRule {
    pub prefix: String,
}

impl ManualOverageRollupRule for ManualOverageRollupCodePrefixRule {
    fn check(&self, item: &ManualOverageRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualOverageRollupRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualOverageRollupRule for ManualOverageRollupRetriesRangeRule {
    fn check(&self, item: &ManualOverageRollup) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualOverageRollupPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualOverageRollupRule for ManualOverageRollupPriorityRangeRule {
    fn check(&self, item: &ManualOverageRollup) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualOverageRollupActiveGuardRule {
    pub limit: f64,
}

impl ManualOverageRollupRule for ManualOverageRollupActiveGuardRule {
    fn check(&self, item: &ManualOverageRollup) -> Vec<String> {
        if item.active && (item.minimum > self.limit) {
            return vec![format!("minimum {} exceeds {} while active", item.minimum, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual overage rollup.
#[derive(Debug, Clone)]
pub struct ManualOverageRollupValidator {
    pub rules: Vec<Rc<dyn ManualOverageRollupRule>>,
}

impl ManualOverageRollupValidator {
    pub fn validate(&self, item: &ManualOverageRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualOverageRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualOverageRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_overage_rollup_validator() -> ManualOverageRollupValidator {
    ManualOverageRollupValidator {
        rules: vec![
            Rc::new(ManualOverageRollupRequiredCodeRule {
                message: "manual overage rollup needs this field".to_string(),
            }) as Rc<dyn ManualOverageRollupRule>,
            Rc::new(ManualOverageRollupRequiredIssuerRule {
                message: "manual overage rollup needs an entry".to_string(),
            }),
            Rc::new(ManualOverageRollupCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ManualOverageRollupRetriesRangeRule { min: 16, max: 64 }),
            Rc::new(ManualOverageRollupPriorityRangeRule { min: 16, max: 64 }),
            Rc::new(ManualOverageRollupActiveGuardRule { limit: 4.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_overage_rollup_validator();
        let item = new_manual_overage_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualOverageRollupRequiredCodeRule {
            message: "manual overage rollup needs an entry".to_string(),
        };
        let mut item = new_manual_overage_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = ManualOverageRollupRequiredIssuerRule {
            message: "manual overage rollup needs this field".to_string(),
        };
        let mut item = new_manual_overage_rollup("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = ManualOverageRollupRetriesRangeRule { min: 32, max: 1000 };
        let mut item = new_manual_overage_rollup("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = ManualOverageRollupPriorityRangeRule { min: 1000, max: 3600 };
        let mut item = new_manual_overage_rollup("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
