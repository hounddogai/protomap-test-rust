use super::*;

use acme_kit::text;

/// Where a lenient escalation rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientEscalationRuleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientEscalationRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientEscalationRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient escalation rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEscalationRule {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub segment: String,
    pub channel: String,
    pub status: LenientEscalationRuleStatus,
}

impl LenientEscalationRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientEscalationRuleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientEscalationRuleStatus) -> bool {
        match self.status {
            LenientEscalationRuleStatus::Created => {
                match next {
                    LenientEscalationRuleStatus::Published | LenientEscalationRuleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEscalationRuleStatus::Validated => {
                return next == LenientEscalationRuleStatus::Published;
            }
            LenientEscalationRuleStatus::Published => {
                match next {
                    LenientEscalationRuleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientEscalationRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient escalation rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> LenientEscalationRule {
        LenientEscalationRule {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            segment: self.segment.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a lenient escalation rule in status created.
pub fn new_lenient_escalation_rule(code: &str, name: &str) -> LenientEscalationRule {
    LenientEscalationRule {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        segment: "".to_string(),
        channel: "".to_string(),
        status: LenientEscalationRuleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_escalation_rule("code-882", "name-228");
        assert_eq!(item.status, LenientEscalationRuleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_lenient_escalation_rule("code-882", "name-228");
        if item.move_to(LenientEscalationRuleStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientEscalationRuleStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_escalation_rule("code-882", "name-228");
        item.status = LenientEscalationRuleStatus::Retired;
        assert!(
            item.move_to(LenientEscalationRuleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
