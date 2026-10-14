use super::*;

use acme_kit::text;

/// Where a runbook slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookSlotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RunbookSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RunbookSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One runbook slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookSlot {
    pub code: String,
    pub label: String,
    pub country: String,
    pub margin: f64,
    pub checksum: String,
    pub audience: String,
    pub locale: String,
    pub tenant_id: String,
    pub fingerprint: String,
    pub status: RunbookSlotStatus,
}

impl RunbookSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookSlotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookSlotStatus) -> bool {
        match self.status {
            RunbookSlotStatus::Idle => {
                return next == RunbookSlotStatus::Draining || next == RunbookSlotStatus::Stopped;
            }
            RunbookSlotStatus::Warming => {
                match next {
                    RunbookSlotStatus::Ready | RunbookSlotStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookSlotStatus::Ready => {
                match next {
                    RunbookSlotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookSlotStatus::Draining => {
                return next == RunbookSlotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> RunbookSlot {
        RunbookSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            margin: self.margin,
            checksum: self.checksum.clone(),
            audience: self.audience.clone(),
            locale: self.locale.clone(),
            tenant_id: self.tenant_id.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a runbook slot in status idle.
pub fn new_runbook_slot(code: &str, label: &str) -> RunbookSlot {
    RunbookSlot {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        margin: 0.0,
        checksum: "".to_string(),
        audience: "".to_string(),
        locale: "".to_string(),
        tenant_id: "".to_string(),
        fingerprint: "".to_string(),
        status: RunbookSlotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_runbook_slot("code-139", "label-859");
        assert_eq!(item.status, RunbookSlotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_runbook_slot("code-139", "label-859");
        if item.move_to(RunbookSlotStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookSlotStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_runbook_slot("code-139", "label-859");
        item.status = RunbookSlotStatus::Stopped;
        assert!(item.move_to(RunbookSlotStatus::Idle).is_err(), "expected move to fail");
    }
}
