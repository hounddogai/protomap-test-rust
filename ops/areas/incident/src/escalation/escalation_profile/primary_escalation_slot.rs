use super::*;

use acme_kit::text;

/// Where a primary escalation slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEscalationSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryEscalationSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PrimaryEscalationSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary escalation slot as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationSlot {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub threshold: f64,
    pub retries: i64,
    pub window_days: i64,
    pub cluster: String,
    pub scope: String,
    pub status: PrimaryEscalationSlotStatus,
}

impl PrimaryEscalationSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEscalationSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEscalationSlotStatus) -> bool {
        match self.status {
            PrimaryEscalationSlotStatus::Planned => {
                match next {
                    PrimaryEscalationSlotStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEscalationSlotStatus::Scheduled => {
                return next == PrimaryEscalationSlotStatus::Completed
                    || next == PrimaryEscalationSlotStatus::Cancelled;
            }
            PrimaryEscalationSlotStatus::InProgress => {
                match next {
                    PrimaryEscalationSlotStatus::Completed | PrimaryEscalationSlotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEscalationSlotStatus::Completed => {
                match next {
                    PrimaryEscalationSlotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PrimaryEscalationSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary escalation slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.retries as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryEscalationSlot {
        PrimaryEscalationSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            threshold: self.threshold,
            retries: self.retries,
            window_days: self.window_days,
            cluster: self.cluster.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary escalation slot in status planned.
pub fn new_primary_escalation_slot(code: &str, name: &str) -> PrimaryEscalationSlot {
    PrimaryEscalationSlot {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        threshold: 0.0,
        retries: 0,
        window_days: 0,
        cluster: "".to_string(),
        scope: "".to_string(),
        status: PrimaryEscalationSlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_escalation_slot("code-339", "name-433");
        assert_eq!(item.status, PrimaryEscalationSlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_primary_escalation_slot("code-339", "name-433");
        if item.move_to(PrimaryEscalationSlotStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryEscalationSlotStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_escalation_slot("code-339", "name-433");
        item.status = PrimaryEscalationSlotStatus::Cancelled;
        assert!(
            item.move_to(PrimaryEscalationSlotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
