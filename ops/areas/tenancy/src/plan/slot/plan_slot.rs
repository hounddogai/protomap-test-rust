use super::*;

use acme_kit::text;

/// Where a plan slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PlanSlotStatus {
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

impl std::fmt::Display for PlanSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanSlot {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub host: String,
    pub updated_at: i64,
    pub limit: i64,
    pub tenant_id: String,
    pub fingerprint: String,
    pub status: PlanSlotStatus,
}

impl PlanSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanSlotStatus) -> bool {
        match self.status {
            PlanSlotStatus::Planned => {
                match next {
                    PlanSlotStatus::Scheduled | PlanSlotStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanSlotStatus::Scheduled => {
                return next == PlanSlotStatus::InProgress || next == PlanSlotStatus::Cancelled;
            }
            PlanSlotStatus::InProgress => {
                return next == PlanSlotStatus::Completed || next == PlanSlotStatus::Cancelled;
            }
            PlanSlotStatus::Completed => {
                match next {
                    PlanSlotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PlanSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.limit as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanSlot {
        PlanSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            host: self.host.clone(),
            updated_at: self.updated_at,
            limit: self.limit,
            tenant_id: self.tenant_id.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a plan slot in status planned.
pub fn new_plan_slot(code: &str, name: &str) -> PlanSlot {
    PlanSlot {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        host: "".to_string(),
        updated_at: 0,
        limit: 0,
        tenant_id: "".to_string(),
        fingerprint: "".to_string(),
        status: PlanSlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_plan_slot("code-559", "name-435");
        assert_eq!(item.status, PlanSlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_plan_slot("code-559", "name-435");
        if item.move_to(PlanSlotStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanSlotStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_plan_slot("code-559", "name-435");
        item.status = PlanSlotStatus::Cancelled;
        assert!(item.move_to(PlanSlotStatus::Planned).is_err(), "expected move to fail");
    }
}
