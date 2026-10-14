use super::*;

use acme_kit::text;

/// Where a failover reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FailoverReservationStatus {
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

impl std::fmt::Display for FailoverReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverReservation {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub floor_value: f64,
    pub reviewer: String,
    pub limit: i64,
    pub status: FailoverReservationStatus,
}

impl FailoverReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverReservationStatus) -> bool {
        match self.status {
            FailoverReservationStatus::New => {
                match next {
                    FailoverReservationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverReservationStatus::Queued => {
                return next == FailoverReservationStatus::Running || next == FailoverReservationStatus::Failed;
            }
            FailoverReservationStatus::Running => {
                match next {
                    FailoverReservationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverReservationStatus::Done => {
                match next {
                    FailoverReservationStatus::Failed => {
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

    pub fn move_to(&mut self, next: FailoverReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.limit as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverReservation {
        FailoverReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            floor_value: self.floor_value,
            reviewer: self.reviewer.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a failover reservation in status new.
pub fn new_failover_reservation(code: &str, title: &str) -> FailoverReservation {
    FailoverReservation {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        floor_value: 0.0,
        reviewer: "".to_string(),
        limit: 0,
        status: FailoverReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_failover_reservation("code-80", "title-706");
        assert_eq!(item.status, FailoverReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_failover_reservation("code-80", "title-706");
        if item.move_to(FailoverReservationStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverReservationStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_failover_reservation("code-80", "title-706");
        item.status = FailoverReservationStatus::Failed;
        assert!(
            item.move_to(FailoverReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
