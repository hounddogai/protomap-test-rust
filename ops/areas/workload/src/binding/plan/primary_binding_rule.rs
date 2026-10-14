use super::*;

use acme_kit::text;

/// Where a primary binding rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryBindingRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryBindingRuleStatus {
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

impl std::fmt::Display for PrimaryBindingRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary binding rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingRule {
    pub code: String,
    pub label: String,
    pub host: String,
    pub principal: String,
    pub source: String,
    pub score: f64,
    pub namespace: String,
    pub weight: f64,
    pub status: PrimaryBindingRuleStatus,
}

impl PrimaryBindingRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryBindingRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryBindingRuleStatus) -> bool {
        match self.status {
            PrimaryBindingRuleStatus::New => {
                return next == PrimaryBindingRuleStatus::Running || next == PrimaryBindingRuleStatus::Done;
            }
            PrimaryBindingRuleStatus::Queued => {
                match next {
                    PrimaryBindingRuleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryBindingRuleStatus::Running => {
                return next == PrimaryBindingRuleStatus::Done || next == PrimaryBindingRuleStatus::Failed;
            }
            PrimaryBindingRuleStatus::Done => {
                return next == PrimaryBindingRuleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryBindingRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary binding rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary binding rule", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.weight + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryBindingRule {
        PrimaryBindingRule {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            principal: self.principal.clone(),
            source: self.source.clone(),
            score: self.score,
            namespace: self.namespace.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary binding rule in status new.
pub fn new_primary_binding_rule(code: &str, label: &str) -> PrimaryBindingRule {
    PrimaryBindingRule {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        principal: "".to_string(),
        source: "".to_string(),
        score: 0.0,
        namespace: "".to_string(),
        weight: 0.0,
        status: PrimaryBindingRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_binding_rule("code-897", "label-782");
        assert_eq!(item.status, PrimaryBindingRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_primary_binding_rule("code-897", "label-782");
        if item.move_to(PrimaryBindingRuleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryBindingRuleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_binding_rule("code-897", "label-782");
        item.status = PrimaryBindingRuleStatus::Failed;
        assert!(
            item.move_to(PrimaryBindingRuleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
