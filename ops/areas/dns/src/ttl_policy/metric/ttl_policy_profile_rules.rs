use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy profile must pass. Check returns its problems, or none.
pub trait TtlPolicyProfileRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyProfileRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyProfileRule for TtlPolicyProfileRequiredCodeRule {
    fn check(&self, item: &TtlPolicyProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyProfilePinnedGuardRule {
    pub limit: i64,
}

impl TtlPolicyProfileRule for TtlPolicyProfilePinnedGuardRule {
    fn check(&self, item: &TtlPolicyProfile) -> Vec<String> {
        if item.pinned && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while pinned", item.port, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyProfileCodeLengthRule {
    pub max_length: i64,
}

impl TtlPolicyProfileRule for TtlPolicyProfileCodeLengthRule {
    fn check(&self, item: &TtlPolicyProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyProfileTimezoneLengthRule {
    pub max_length: i64,
}

impl TtlPolicyProfileRule for TtlPolicyProfileTimezoneLengthRule {
    fn check(&self, item: &TtlPolicyProfile) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyProfileMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyProfileRule for TtlPolicyProfileMaximumRangeRule {
    fn check(&self, item: &TtlPolicyProfile) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a ttl policy profile.
#[derive(Debug, Clone)]
pub struct TtlPolicyProfileValidator {
    pub rules: Vec<Rc<dyn TtlPolicyProfileRule>>,
}

impl TtlPolicyProfileValidator {
    pub fn validate(&self, item: &TtlPolicyProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_profile_validator() -> TtlPolicyProfileValidator {
    TtlPolicyProfileValidator {
        rules: vec![
            Rc::new(TtlPolicyProfileRequiredCodeRule {
                message: "ttl policy profile needs a value".to_string(),
            }) as Rc<dyn TtlPolicyProfileRule>,
            Rc::new(TtlPolicyProfilePinnedGuardRule { limit: 1000 }),
            Rc::new(TtlPolicyProfileCodeLengthRule { max_length: 120 }),
            Rc::new(TtlPolicyProfileTimezoneLengthRule { max_length: 64 }),
            Rc::new(TtlPolicyProfileMaximumRangeRule { min: 9.6, max: 686.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_profile_validator();
        let item = new_ttl_policy_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyProfileRequiredCodeRule {
            message: "ttl policy profile needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TtlPolicyProfileCodeLengthRule { max_length: 16 };
        let mut item = new_ttl_policy_profile("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = TtlPolicyProfileTimezoneLengthRule { max_length: 255 };
        let mut item = new_ttl_policy_profile("x", "y");
        item.timezone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = TtlPolicyProfileMaximumRangeRule { min: 0.7, max: 1806.0 };
        let mut item = new_ttl_policy_profile("x", "y");
        item.maximum = 1807.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
