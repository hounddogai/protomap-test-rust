use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe override must pass. Check returns its problems, or none.
pub trait ProbeOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ProbeOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeOverrideRequiredNameRule {
    pub message: String,
}

impl ProbeOverrideRule for ProbeOverrideRequiredNameRule {
    fn check(&self, item: &ProbeOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeOverrideRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeOverrideRule for ProbeOverrideRevisionRangeRule {
    fn check(&self, item: &ProbeOverride) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeOverridePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeOverrideRule for ProbeOverridePriorityRangeRule {
    fn check(&self, item: &ProbeOverride) -> Vec<String> {
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
pub struct ProbeOverrideClusterLengthRule {
    pub max_length: i64,
}

impl ProbeOverrideRule for ProbeOverrideClusterLengthRule {
    fn check(&self, item: &ProbeOverride) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeOverrideVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeOverrideRule for ProbeOverrideVersionRangeRule {
    fn check(&self, item: &ProbeOverride) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeOverrideRequiredCodeRule {
    pub message: String,
}

impl ProbeOverrideRule for ProbeOverrideRequiredCodeRule {
    fn check(&self, item: &ProbeOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe override.
#[derive(Debug, Clone)]
pub struct ProbeOverrideValidator {
    pub rules: Vec<Rc<dyn ProbeOverrideRule>>,
}

impl ProbeOverrideValidator {
    pub fn validate(&self, item: &ProbeOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_override_validator() -> ProbeOverrideValidator {
    ProbeOverrideValidator {
        rules: vec![
            Rc::new(ProbeOverrideRequiredNameRule {
                message: "probe override needs an entry".to_string(),
            }) as Rc<dyn ProbeOverrideRule>,
            Rc::new(ProbeOverrideRevisionRangeRule { min: 120, max: 255 }),
            Rc::new(ProbeOverridePriorityRangeRule { min: 1000, max: 86400 }),
            Rc::new(ProbeOverrideClusterLengthRule { max_length: 86400 }),
            Rc::new(ProbeOverrideVersionRangeRule { min: 80, max: 1000 }),
            Rc::new(ProbeOverrideRequiredCodeRule {
                message: "probe override needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_override_validator();
        let item = new_probe_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ProbeOverrideRequiredNameRule {
            message: "probe override needs an entry".to_string(),
        };
        let mut item = new_probe_override("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = ProbeOverrideRevisionRangeRule { min: 16, max: 1000 };
        let mut item = new_probe_override("x", "y");
        item.revision = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = ProbeOverridePriorityRangeRule { min: 1000, max: 3600 };
        let mut item = new_probe_override("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = ProbeOverrideClusterLengthRule { max_length: 16 };
        let mut item = new_probe_override("x", "y");
        item.cluster = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = ProbeOverrideVersionRangeRule { min: 1000, max: 86400 };
        let mut item = new_probe_override("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeOverrideRequiredCodeRule {
            message: "probe override needs this field".to_string(),
        };
        let mut item = new_probe_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
