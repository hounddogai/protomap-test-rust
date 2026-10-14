use super::*;

use acme_kit::text;

/// Where a audit event cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AuditEventCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AuditEventCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a audit event cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCycle {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub visible: bool,
    pub title: String,
    pub budget: f64,
    pub region: String,
    pub status: AuditEventCycleStatus,
}

impl AuditEventCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventCycleStatus) -> bool {
        match self.status {
            AuditEventCycleStatus::Created => {
                match next {
                    AuditEventCycleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventCycleStatus::Validated => {
                match next {
                    AuditEventCycleStatus::Published | AuditEventCycleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventCycleStatus::Published => {
                return next == AuditEventCycleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.sequence as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventCycle {
        AuditEventCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            visible: self.visible,
            title: self.title.clone(),
            budget: self.budget,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a audit event cycle in status created.
pub fn new_audit_event_cycle(code: &str, label: &str) -> AuditEventCycle {
    AuditEventCycle {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        visible: false,
        title: "".to_string(),
        budget: 0.0,
        region: "".to_string(),
        status: AuditEventCycleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_audit_event_cycle("code-526", "label-698");
        assert_eq!(item.status, AuditEventCycleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_audit_event_cycle("code-526", "label-698");
        if item.move_to(AuditEventCycleStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventCycleStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_audit_event_cycle("code-526", "label-698");
        item.status = AuditEventCycleStatus::Retired;
        assert!(
            item.move_to(AuditEventCycleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
