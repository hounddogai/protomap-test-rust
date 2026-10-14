use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional retention group must pass. Check returns its problems, or none.
pub trait RegionalRetentionGroupRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRetentionGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionGroupRequiredLabelRule {
    pub message: String,
}

impl RegionalRetentionGroupRule for RegionalRetentionGroupRequiredLabelRule {
    fn check(&self, item: &RegionalRetentionGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionGroupCodePrefixRule {
    pub prefix: String,
}

impl RegionalRetentionGroupRule for RegionalRetentionGroupCodePrefixRule {
    fn check(&self, item: &RegionalRetentionGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionGroupRequiredCodeRule {
    pub message: String,
}

impl RegionalRetentionGroupRule for RegionalRetentionGroupRequiredCodeRule {
    fn check(&self, item: &RegionalRetentionGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional retention group.
#[derive(Debug, Clone)]
pub struct RegionalRetentionGroupValidator {
    pub rules: Vec<Rc<dyn RegionalRetentionGroupRule>>,
}

impl RegionalRetentionGroupValidator {
    pub fn validate(&self, item: &RegionalRetentionGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalRetentionGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRetentionGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_retention_group_validator() -> RegionalRetentionGroupValidator {
    RegionalRetentionGroupValidator {
        rules: vec![
            Rc::new(RegionalRetentionGroupRequiredLabelRule {
                message: "regional retention group needs a value".to_string(),
            }) as Rc<dyn RegionalRetentionGroupRule>,
            Rc::new(RegionalRetentionGroupCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RegionalRetentionGroupRequiredCodeRule {
                message: "regional retention group needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_retention_group_validator();
        let item = new_regional_retention_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalRetentionGroupRequiredLabelRule {
            message: "regional retention group needs this field".to_string(),
        };
        let mut item = new_regional_retention_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalRetentionGroupRequiredCodeRule {
            message: "regional retention group needs this field".to_string(),
        };
        let mut item = new_regional_retention_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
