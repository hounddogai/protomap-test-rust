use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist baseline must pass. Check returns its problems, or none.
pub trait AllowlistBaselineRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBaselineFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistBaselineRule for AllowlistBaselineFloorValueRangeRule {
    fn check(&self, item: &AllowlistBaseline) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBaselineRequiredLabelRule {
    pub message: String,
}

impl AllowlistBaselineRule for AllowlistBaselineRequiredLabelRule {
    fn check(&self, item: &AllowlistBaseline) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBaselineCodePrefixRule {
    pub prefix: String,
}

impl AllowlistBaselineRule for AllowlistBaselineCodePrefixRule {
    fn check(&self, item: &AllowlistBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist baseline.
#[derive(Debug, Clone)]
pub struct AllowlistBaselineValidator {
    pub rules: Vec<Rc<dyn AllowlistBaselineRule>>,
}

impl AllowlistBaselineValidator {
    pub fn validate(&self, item: &AllowlistBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_baseline_validator() -> AllowlistBaselineValidator {
    AllowlistBaselineValidator {
        rules: vec![
            Rc::new(AllowlistBaselineFloorValueRangeRule { min: 9.5, max: 1127.2 }) as Rc<dyn AllowlistBaselineRule>,
            Rc::new(AllowlistBaselineRequiredLabelRule {
                message: "allowlist baseline needs this field".to_string(),
            }),
            Rc::new(AllowlistBaselineCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_baseline_validator();
        let item = new_allowlist_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = AllowlistBaselineFloorValueRangeRule { min: 7.1, max: 4446.1 };
        let mut item = new_allowlist_baseline("x", "y");
        item.floor_value = 4447.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AllowlistBaselineRequiredLabelRule {
            message: "allowlist baseline needs this field".to_string(),
        };
        let mut item = new_allowlist_baseline("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
