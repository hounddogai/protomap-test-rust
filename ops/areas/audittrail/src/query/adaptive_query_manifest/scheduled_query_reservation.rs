use super::*;

use acme_kit::text;

/// Where a scheduled query reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledQueryReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledQueryReservationStatus {
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

impl std::fmt::Display for ScheduledQueryReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled query reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryReservation {
    pub code: String,
    pub label: String,
    pub description: String,
    pub duration_minutes: i64,
    pub weight: f64,
    pub minimum: f64,
    pub reviewer: String,
    pub burst: i64,
    pub status: ScheduledQueryReservationStatus,
}

impl ScheduledQueryReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledQueryReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledQueryReservationStatus) -> bool {
        match self.status {
            ScheduledQueryReservationStatus::Planned => {
                match next {
                    ScheduledQueryReservationStatus::InProgress | ScheduledQueryReservationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQueryReservationStatus::Scheduled => {
                match next {
                    ScheduledQueryReservationStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQueryReservationStatus::InProgress => {
                return next == ScheduledQueryReservationStatus::Completed;
            }
            ScheduledQueryReservationStatus::Completed => {
                match next {
                    ScheduledQueryReservationStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ScheduledQueryReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled query reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        (self.minimum * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledQueryReservation {
        ScheduledQueryReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            duration_minutes: self.duration_minutes,
            weight: self.weight,
            minimum: self.minimum,
            reviewer: self.reviewer.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled query reservation in status planned.
pub fn new_scheduled_query_reservation(code: &str, label: &str) -> ScheduledQueryReservation {
    ScheduledQueryReservation {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        duration_minutes: 0,
        weight: 0.0,
        minimum: 0.0,
        reviewer: "".to_string(),
        burst: 0,
        status: ScheduledQueryReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_scheduled_query_reservation("code-525", "label-982");
        assert_eq!(item.status, ScheduledQueryReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_scheduled_query_reservation("code-525", "label-982");
        if item.move_to(ScheduledQueryReservationStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledQueryReservationStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_scheduled_query_reservation("code-525", "label-982");
        item.status = ScheduledQueryReservationStatus::Cancelled;
        assert!(
            item.move_to(ScheduledQueryReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
