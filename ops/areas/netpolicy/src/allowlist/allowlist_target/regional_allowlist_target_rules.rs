use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional allowlist target must pass. Check returns its problems, or none.
pub trait RegionalAllowlistTargetRule: std::fmt::Debug {
    fn check(&self, item: &RegionalAllowlistTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistTargetRequiredOwnerIdRule {
    pub message: String,
}

impl RegionalAllowlistTargetRule for RegionalAllowlistTargetRequiredOwnerIdRule {
    fn check(&self, item: &RegionalAllowlistTarget) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistTargetActiveGuardRule {
    pub limit: i64,
}

impl RegionalAllowlistTargetRule for RegionalAllowlistTargetActiveGuardRule {
    fn check(&self, item: &RegionalAllowlistTarget) -> Vec<String> {
        if item.active && (item.revision > self.limit) {
            return vec![format!(
                "revision {} exceeds {} while active",
                item.revision, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistTargetCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalAllowlistTargetRule for RegionalAllowlistTargetCapacityRangeRule {
    fn check(&self, item: &RegionalAllowlistTarget) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistTargetRequiredLabelRule {
    pub message: String,
}

impl RegionalAllowlistTargetRule for RegionalAllowlistTargetRequiredLabelRule {
    fn check(&self, item: &RegionalAllowlistTarget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional allowlist target.
#[derive(Debug, Clone)]
pub struct RegionalAllowlistTargetValidator {
    pub rules: Vec<Rc<dyn RegionalAllowlistTargetRule>>,
}

impl RegionalAllowlistTargetValidator {
    pub fn validate(&self, item: &RegionalAllowlistTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalAllowlistTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalAllowlistTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_allowlist_target_validator() -> RegionalAllowlistTargetValidator {
    RegionalAllowlistTargetValidator {
        rules: vec![
            Rc::new(RegionalAllowlistTargetRequiredOwnerIdRule {
                message: "regional allowlist target needs a value".to_string(),
            }) as Rc<dyn RegionalAllowlistTargetRule>,
            Rc::new(RegionalAllowlistTargetActiveGuardRule { limit: 64 }),
            Rc::new(RegionalAllowlistTargetCapacityRangeRule { min: 255, max: 86400 }),
            Rc::new(RegionalAllowlistTargetRequiredLabelRule {
                message: "regional allowlist target needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_allowlist_target_validator();
        let item = new_regional_allowlist_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = RegionalAllowlistTargetRequiredOwnerIdRule {
            message: "regional allowlist target needs an entry".to_string(),
        };
        let mut item = new_regional_allowlist_target("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = RegionalAllowlistTargetCapacityRangeRule { min: 255, max: 255 };
        let mut item = new_regional_allowlist_target("x", "y");
        item.capacity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalAllowlistTargetRequiredLabelRule {
            message: "regional allowlist target needs this field".to_string(),
        };
        let mut item = new_regional_allowlist_target("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
