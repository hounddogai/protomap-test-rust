use super::*;

use acme_kit::text;

/// Where a runbook reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RunbookReservationStatus {
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

impl std::fmt::Display for RunbookReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReservation {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub weight: f64,
    pub title: String,
    pub contact_email: String,
    pub status: RunbookReservationStatus,
}

impl RunbookReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookReservationStatus) -> bool {
        match self.status {
            RunbookReservationStatus::Idle => {
                return next == RunbookReservationStatus::Draining;
            }
            RunbookReservationStatus::Warming => {
                return next == RunbookReservationStatus::Draining || next == RunbookReservationStatus::Stopped;
            }
            RunbookReservationStatus::Ready => {
                return next == RunbookReservationStatus::Stopped;
            }
            RunbookReservationStatus::Draining => {
                match next {
                    RunbookReservationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RunbookReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.weight * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookReservation {
        RunbookReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            weight: self.weight,
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a runbook reservation in status idle.
pub fn new_runbook_reservation(code: &str, name: &str) -> RunbookReservation {
    RunbookReservation {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        weight: 0.0,
        title: "".to_string(),
        contact_email: "".to_string(),
        status: RunbookReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_runbook_reservation("code-465", "name-62");
        assert_eq!(item.status, RunbookReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_runbook_reservation("code-465", "name-62");
        if item.move_to(RunbookReservationStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookReservationStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_runbook_reservation("code-465", "name-62");
        item.status = RunbookReservationStatus::Stopped;
        assert!(
            item.move_to(RunbookReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
