use super::*;

use acme_kit::text;

/// Where a egress rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EgressRuleStatus {
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

impl std::fmt::Display for EgressRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressRule {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub visible: bool,
    pub active: bool,
    pub summary: String,
    pub channel: String,
    pub discount: f64,
    pub rate: f64,
    pub category: String,
    pub quantity: i64,
    pub status: EgressRuleStatus,
}

impl EgressRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressRuleStatus) -> bool {
        match self.status {
            EgressRuleStatus::New => {
                match next {
                    EgressRuleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressRuleStatus::Queued => {
                match next {
                    EgressRuleStatus::Running | EgressRuleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressRuleStatus::Running => {
                match next {
                    EgressRuleStatus::Done | EgressRuleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressRuleStatus::Done => {
                return next == EgressRuleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "egress rule", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.rate + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressRule {
        EgressRule {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            visible: self.visible,
            active: self.active,
            summary: self.summary.clone(),
            channel: self.channel.clone(),
            discount: self.discount,
            rate: self.rate,
            category: self.category.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a egress rule in status new.
pub fn new_egress_rule(code: &str, label: &str) -> EgressRule {
    EgressRule {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        visible: false,
        active: false,
        summary: "".to_string(),
        channel: "".to_string(),
        discount: 0.0,
        rate: 0.0,
        category: "".to_string(),
        quantity: 0,
        status: EgressRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_egress_rule("code-842", "label-743");
        assert_eq!(item.status, EgressRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_egress_rule("code-842", "label-743");
        if item.move_to(EgressRuleStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressRuleStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_egress_rule("code-842", "label-743");
        item.status = EgressRuleStatus::Failed;
        assert!(item.move_to(EgressRuleStatus::New).is_err(), "expected move to fail");
    }
}
