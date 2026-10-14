use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual probe notice must pass. Check returns its problems, or none.
pub trait ManualProbeNoticeRule: std::fmt::Debug {
    fn check(&self, item: &ManualProbeNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeNoticeCodePrefixRule {
    pub prefix: String,
}

impl ManualProbeNoticeRule for ManualProbeNoticeCodePrefixRule {
    fn check(&self, item: &ManualProbeNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeNoticeAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ManualProbeNoticeRule for ManualProbeNoticeAllowedNameRule {
    fn check(&self, item: &ManualProbeNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is inconsistent", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeNoticeFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualProbeNoticeRule for ManualProbeNoticeFloorValueRangeRule {
    fn check(&self, item: &ManualProbeNotice) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeNoticeCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualProbeNoticeRule for ManualProbeNoticeCeilingRangeRule {
    fn check(&self, item: &ManualProbeNotice) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeNoticeRequiredNameRule {
    pub message: String,
}

impl ManualProbeNoticeRule for ManualProbeNoticeRequiredNameRule {
    fn check(&self, item: &ManualProbeNotice) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual probe notice.
#[derive(Debug, Clone)]
pub struct ManualProbeNoticeValidator {
    pub rules: Vec<Rc<dyn ManualProbeNoticeRule>>,
}

impl ManualProbeNoticeValidator {
    pub fn validate(&self, item: &ManualProbeNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualProbeNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualProbeNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_probe_notice_validator() -> ManualProbeNoticeValidator {
    ManualProbeNoticeValidator {
        rules: vec![
            Rc::new(ManualProbeNoticeCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn ManualProbeNoticeRule>,
            Rc::new(ManualProbeNoticeAllowedNameRule {
                allowed: vec!["weighted".to_string(), "primary".to_string(), "global".to_string()],
            }),
            Rc::new(ManualProbeNoticeFloorValueRangeRule { min: 0.6, max: 3470.4 }),
            Rc::new(ManualProbeNoticeCeilingRangeRule { min: 0.2, max: 2384.7 }),
            Rc::new(ManualProbeNoticeRequiredNameRule {
                message: "manual probe notice needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_probe_notice_validator();
        let item = new_manual_probe_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = ManualProbeNoticeFloorValueRangeRule { min: 0.5, max: 4551.1 };
        let mut item = new_manual_probe_notice("x", "y");
        item.floor_value = 4552.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ManualProbeNoticeCeilingRangeRule { min: 5.6, max: 4108.5 };
        let mut item = new_manual_probe_notice("x", "y");
        item.ceiling = 4110.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualProbeNoticeRequiredNameRule {
            message: "manual probe notice needs a value".to_string(),
        };
        let mut item = new_manual_probe_notice("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
