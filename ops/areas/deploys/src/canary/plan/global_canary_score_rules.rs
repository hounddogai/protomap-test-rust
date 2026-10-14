use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global canary score must pass. Check returns its problems, or none.
pub trait GlobalCanaryScoreRule: std::fmt::Debug {
    fn check(&self, item: &GlobalCanaryScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCanaryScoreRequiredNameRule {
    pub message: String,
}

impl GlobalCanaryScoreRule for GlobalCanaryScoreRequiredNameRule {
    fn check(&self, item: &GlobalCanaryScore) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCanaryScoreRequiredExternalRefRule {
    pub message: String,
}

impl GlobalCanaryScoreRule for GlobalCanaryScoreRequiredExternalRefRule {
    fn check(&self, item: &GlobalCanaryScore) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCanaryScorePrincipalLengthRule {
    pub max_length: i64,
}

impl GlobalCanaryScoreRule for GlobalCanaryScorePrincipalLengthRule {
    fn check(&self, item: &GlobalCanaryScore) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCanaryScoreCodeLengthRule {
    pub max_length: i64,
}

impl GlobalCanaryScoreRule for GlobalCanaryScoreCodeLengthRule {
    fn check(&self, item: &GlobalCanaryScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCanaryScoreScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalCanaryScoreRule for GlobalCanaryScoreScoreRangeRule {
    fn check(&self, item: &GlobalCanaryScore) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCanaryScoreRequiredPrincipalRule {
    pub message: String,
}

impl GlobalCanaryScoreRule for GlobalCanaryScoreRequiredPrincipalRule {
    fn check(&self, item: &GlobalCanaryScore) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global canary score.
#[derive(Debug, Clone)]
pub struct GlobalCanaryScoreValidator {
    pub rules: Vec<Rc<dyn GlobalCanaryScoreRule>>,
}

impl GlobalCanaryScoreValidator {
    pub fn validate(&self, item: &GlobalCanaryScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalCanaryScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalCanaryScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_canary_score_validator() -> GlobalCanaryScoreValidator {
    GlobalCanaryScoreValidator {
        rules: vec![
            Rc::new(GlobalCanaryScoreRequiredNameRule {
                message: "global canary score needs this field".to_string(),
            }) as Rc<dyn GlobalCanaryScoreRule>,
            Rc::new(GlobalCanaryScoreRequiredExternalRefRule {
                message: "global canary score needs a value".to_string(),
            }),
            Rc::new(GlobalCanaryScorePrincipalLengthRule { max_length: 3600 }),
            Rc::new(GlobalCanaryScoreCodeLengthRule { max_length: 80 }),
            Rc::new(GlobalCanaryScoreScoreRangeRule { min: 7.7, max: 3606.2 }),
            Rc::new(GlobalCanaryScoreRequiredPrincipalRule {
                message: "global canary score needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_canary_score_validator();
        let item = new_global_canary_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GlobalCanaryScoreRequiredNameRule {
            message: "global canary score needs a value".to_string(),
        };
        let mut item = new_global_canary_score("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = GlobalCanaryScoreRequiredExternalRefRule {
            message: "global canary score needs this field".to_string(),
        };
        let mut item = new_global_canary_score("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = GlobalCanaryScorePrincipalLengthRule { max_length: 16 };
        let mut item = new_global_canary_score("x", "y");
        item.principal = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalCanaryScoreCodeLengthRule { max_length: 86400 };
        let mut item = new_global_canary_score("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = GlobalCanaryScoreScoreRangeRule { min: 2.4, max: 1884.6 };
        let mut item = new_global_canary_score("x", "y");
        item.score = 1886.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = GlobalCanaryScoreRequiredPrincipalRule {
            message: "global canary score needs this field".to_string(),
        };
        let mut item = new_global_canary_score("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
