use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a escalation cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationCycleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EscalationCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for EscalationCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation cycle as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationCycle {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub created_at: i64,
    pub scope: String,
    pub principal: String,
    pub attributes: BTreeMap<String, String>,
    pub label: String,
    pub status: EscalationCycleStatus,
}

impl EscalationCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationCycleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationCycleStatus) -> bool {
        match self.status {
            EscalationCycleStatus::Open => {
                match next {
                    EscalationCycleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationCycleStatus::InReview => {
                match next {
                    EscalationCycleStatus::Resolved | EscalationCycleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationCycleStatus::Resolved => {
                match next {
                    EscalationCycleStatus::Reopened | EscalationCycleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationCycleStatus::Reopened => {
                return next == EscalationCycleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> EscalationCycle {
        EscalationCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            created_at: self.created_at,
            scope: self.scope.clone(),
            principal: self.principal.clone(),
            attributes: self.attributes.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a escalation cycle in status open.
pub fn new_escalation_cycle(code: &str, name: &str) -> EscalationCycle {
    EscalationCycle {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        created_at: 0,
        scope: "".to_string(),
        principal: "".to_string(),
        attributes: BTreeMap::new(),
        label: "".to_string(),
        status: EscalationCycleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_escalation_cycle("code-919", "name-270");
        assert_eq!(item.status, EscalationCycleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_escalation_cycle("code-919", "name-270");
        if item.move_to(EscalationCycleStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationCycleStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_escalation_cycle("code-919", "name-270");
        item.status = EscalationCycleStatus::Archived;
        assert!(
            item.move_to(EscalationCycleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
