use super::*;

use acme_kit::text;

/// Where a probe rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ProbeRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ProbeRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe rule as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRule {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub issuer: String,
    pub region: String,
    pub archived: bool,
    pub currency: String,
    pub amount: f64,
    pub status: ProbeRuleStatus,
}

impl ProbeRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeRuleStatus) -> bool {
        match self.status {
            ProbeRuleStatus::New => {
                match next {
                    ProbeRuleStatus::Queued | ProbeRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeRuleStatus::Queued => {
                return next == ProbeRuleStatus::Done;
            }
            ProbeRuleStatus::Running => {
                return next == ProbeRuleStatus::Done;
            }
            ProbeRuleStatus::Done => {
                match next {
                    ProbeRuleStatus::Failed => {
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

    pub fn move_to(&mut self, next: ProbeRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> ProbeRule {
        ProbeRule {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            issuer: self.issuer.clone(),
            region: self.region.clone(),
            archived: self.archived,
            currency: self.currency.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a probe rule in status new.
pub fn new_probe_rule(code: &str, name: &str) -> ProbeRule {
    ProbeRule {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        issuer: "".to_string(),
        region: "".to_string(),
        archived: false,
        currency: "".to_string(),
        amount: 0.0,
        status: ProbeRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_probe_rule("code-274", "name-531");
        assert_eq!(item.status, ProbeRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_probe_rule("code-274", "name-531");
        if item.move_to(ProbeRuleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeRuleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_probe_rule("code-274", "name-531");
        item.status = ProbeRuleStatus::Failed;
        assert!(item.move_to(ProbeRuleStatus::New).is_err(), "expected move to fail");
    }
}
