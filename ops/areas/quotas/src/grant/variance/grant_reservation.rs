use super::*;

use acme_kit::text;

/// Where a grant reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GrantReservationStatus {
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

impl std::fmt::Display for GrantReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One grant reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantReservation {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub host: String,
    pub region: String,
    pub quantity: i64,
    pub status: GrantReservationStatus,
}

impl GrantReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantReservationStatus) -> bool {
        match self.status {
            GrantReservationStatus::New => {
                return next == GrantReservationStatus::Done;
            }
            GrantReservationStatus::Queued => {
                match next {
                    GrantReservationStatus::Done | GrantReservationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantReservationStatus::Running => {
                return next == GrantReservationStatus::Done;
            }
            GrantReservationStatus::Done => {
                match next {
                    GrantReservationStatus::Failed => {
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

    pub fn move_to(&mut self, next: GrantReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant reservation {} cannot move from {} to {}",
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
        self.quantity as f64
    }

    pub fn copy(&self) -> GrantReservation {
        GrantReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            host: self.host.clone(),
            region: self.region.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a grant reservation in status new.
pub fn new_grant_reservation(code: &str, name: &str) -> GrantReservation {
    GrantReservation {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        host: "".to_string(),
        region: "".to_string(),
        quantity: 0,
        status: GrantReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_grant_reservation("code-751", "name-467");
        assert_eq!(item.status, GrantReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_grant_reservation("code-751", "name-467");
        if item.move_to(GrantReservationStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantReservationStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_grant_reservation("code-751", "name-467");
        item.status = GrantReservationStatus::Failed;
        assert!(
            item.move_to(GrantReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
