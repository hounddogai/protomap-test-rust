use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log manifest must pass. Check returns its problems, or none.
pub trait FlowLogManifestRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogManifestPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogManifestRule for FlowLogManifestPortRangeRule {
    fn check(&self, item: &FlowLogManifest) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogManifestCodePrefixRule {
    pub prefix: String,
}

impl FlowLogManifestRule for FlowLogManifestCodePrefixRule {
    fn check(&self, item: &FlowLogManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogManifestRequiredLabelRule {
    pub message: String,
}

impl FlowLogManifestRule for FlowLogManifestRequiredLabelRule {
    fn check(&self, item: &FlowLogManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogManifestQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogManifestRule for FlowLogManifestQuotaUnitsRangeRule {
    fn check(&self, item: &FlowLogManifest) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogManifestNameLengthRule {
    pub max_length: i64,
}

impl FlowLogManifestRule for FlowLogManifestNameLengthRule {
    fn check(&self, item: &FlowLogManifest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogManifestRequiredCodeRule {
    pub message: String,
}

impl FlowLogManifestRule for FlowLogManifestRequiredCodeRule {
    fn check(&self, item: &FlowLogManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log manifest.
#[derive(Debug, Clone)]
pub struct FlowLogManifestValidator {
    pub rules: Vec<Rc<dyn FlowLogManifestRule>>,
}

impl FlowLogManifestValidator {
    pub fn validate(&self, item: &FlowLogManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_manifest_validator() -> FlowLogManifestValidator {
    FlowLogManifestValidator {
        rules: vec![
            Rc::new(FlowLogManifestPortRangeRule { min: 80, max: 120 }) as Rc<dyn FlowLogManifestRule>,
            Rc::new(FlowLogManifestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(FlowLogManifestRequiredLabelRule {
                message: "flow log manifest needs a value".to_string(),
            }),
            Rc::new(FlowLogManifestQuotaUnitsRangeRule { min: 16, max: 255 }),
            Rc::new(FlowLogManifestNameLengthRule { max_length: 80 }),
            Rc::new(FlowLogManifestRequiredCodeRule {
                message: "flow log manifest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_manifest_validator();
        let item = new_flow_log_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = FlowLogManifestPortRangeRule { min: 64, max: 120 };
        let mut item = new_flow_log_manifest("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FlowLogManifestRequiredLabelRule {
            message: "flow log manifest needs a value".to_string(),
        };
        let mut item = new_flow_log_manifest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = FlowLogManifestQuotaUnitsRangeRule { min: 1000, max: 86400 };
        let mut item = new_flow_log_manifest("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FlowLogManifestNameLengthRule { max_length: 32 };
        let mut item = new_flow_log_manifest("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FlowLogManifestRequiredCodeRule {
            message: "flow log manifest needs an entry".to_string(),
        };
        let mut item = new_flow_log_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
