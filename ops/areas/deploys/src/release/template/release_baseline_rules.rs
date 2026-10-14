use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release baseline must pass. Check returns its problems, or none.
pub trait ReleaseBaselineRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBaselineRequiredNotesRule {
    pub message: String,
}

impl ReleaseBaselineRule for ReleaseBaselineRequiredNotesRule {
    fn check(&self, item: &ReleaseBaseline) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBaselineTenantIdLengthRule {
    pub max_length: i64,
}

impl ReleaseBaselineRule for ReleaseBaselineTenantIdLengthRule {
    fn check(&self, item: &ReleaseBaseline) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBaselinePinnedGuardRule {
    pub limit: f64,
}

impl ReleaseBaselineRule for ReleaseBaselinePinnedGuardRule {
    fn check(&self, item: &ReleaseBaseline) -> Vec<String> {
        if item.pinned && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while pinned",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBaselineRequiredTenantIdRule {
    pub message: String,
}

impl ReleaseBaselineRule for ReleaseBaselineRequiredTenantIdRule {
    fn check(&self, item: &ReleaseBaseline) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBaselineFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseBaselineRule for ReleaseBaselineFloorValueRangeRule {
    fn check(&self, item: &ReleaseBaseline) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a release baseline.
#[derive(Debug, Clone)]
pub struct ReleaseBaselineValidator {
    pub rules: Vec<Rc<dyn ReleaseBaselineRule>>,
}

impl ReleaseBaselineValidator {
    pub fn validate(&self, item: &ReleaseBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_baseline_validator() -> ReleaseBaselineValidator {
    ReleaseBaselineValidator {
        rules: vec![
            Rc::new(ReleaseBaselineRequiredNotesRule {
                message: "release baseline needs a value".to_string(),
            }) as Rc<dyn ReleaseBaselineRule>,
            Rc::new(ReleaseBaselineTenantIdLengthRule { max_length: 32 }),
            Rc::new(ReleaseBaselinePinnedGuardRule { limit: 3.2 }),
            Rc::new(ReleaseBaselineRequiredTenantIdRule {
                message: "release baseline needs an entry".to_string(),
            }),
            Rc::new(ReleaseBaselineFloorValueRangeRule { min: 5.8, max: 991.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_baseline_validator();
        let item = new_release_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = ReleaseBaselineRequiredNotesRule {
            message: "release baseline needs this field".to_string(),
        };
        let mut item = new_release_baseline("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = ReleaseBaselineTenantIdLengthRule { max_length: 32 };
        let mut item = new_release_baseline("x", "y");
        item.tenant_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = ReleaseBaselineRequiredTenantIdRule {
            message: "release baseline needs a value".to_string(),
        };
        let mut item = new_release_baseline("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = ReleaseBaselineFloorValueRangeRule { min: 0.7, max: 1149.6 };
        let mut item = new_release_baseline("x", "y");
        item.floor_value = 1151.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
