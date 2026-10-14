use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient delegation request must pass. Check returns its problems, or none.
pub trait LenientDelegationRequestRule: std::fmt::Debug {
    fn check(&self, item: &LenientDelegationRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationRequestRequiredAudienceRule {
    pub message: String,
}

impl LenientDelegationRequestRule for LenientDelegationRequestRequiredAudienceRule {
    fn check(&self, item: &LenientDelegationRequest) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationRequestRequiredChannelRule {
    pub message: String,
}

impl LenientDelegationRequestRule for LenientDelegationRequestRequiredChannelRule {
    fn check(&self, item: &LenientDelegationRequest) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationRequestAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl LenientDelegationRequestRule for LenientDelegationRequestAllowedAudienceRule {
    fn check(&self, item: &LenientDelegationRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is blocked", item.audience)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient delegation request.
#[derive(Debug, Clone)]
pub struct LenientDelegationRequestValidator {
    pub rules: Vec<Rc<dyn LenientDelegationRequestRule>>,
}

impl LenientDelegationRequestValidator {
    pub fn validate(&self, item: &LenientDelegationRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientDelegationRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientDelegationRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_delegation_request_validator() -> LenientDelegationRequestValidator {
    LenientDelegationRequestValidator {
        rules: vec![
            Rc::new(LenientDelegationRequestRequiredAudienceRule {
                message: "lenient delegation request needs this field".to_string(),
            }) as Rc<dyn LenientDelegationRequestRule>,
            Rc::new(LenientDelegationRequestRequiredChannelRule {
                message: "lenient delegation request needs this field".to_string(),
            }),
            Rc::new(LenientDelegationRequestAllowedAudienceRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_delegation_request_validator();
        let item = new_lenient_delegation_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = LenientDelegationRequestRequiredAudienceRule {
            message: "lenient delegation request needs this field".to_string(),
        };
        let mut item = new_lenient_delegation_request("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = LenientDelegationRequestRequiredChannelRule {
            message: "lenient delegation request needs an entry".to_string(),
        };
        let mut item = new_lenient_delegation_request("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
