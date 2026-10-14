use super::*;

use acme_kit::text;

/// Where a reservation metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationMetricStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ReservationMetricStatus {
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

impl std::fmt::Display for ReservationMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a reservation metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMetric {
    pub code: String,
    pub label: String,
    pub host: String,
    pub key_id: String,
    pub amount: f64,
    pub cost: f64,
    pub ratio: f64,
    pub status: ReservationMetricStatus,
}

impl ReservationMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationMetricStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationMetricStatus) -> bool {
        match self.status {
            ReservationMetricStatus::Planned => {
                return next == ReservationMetricStatus::InProgress || next == ReservationMetricStatus::Cancelled;
            }
            ReservationMetricStatus::Scheduled => {
                match next {
                    ReservationMetricStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationMetricStatus::InProgress => {
                return next == ReservationMetricStatus::Completed || next == ReservationMetricStatus::Cancelled;
            }
            ReservationMetricStatus::Completed => {
                return next == ReservationMetricStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.cost - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationMetric {
        ReservationMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            key_id: self.key_id.clone(),
            amount: self.amount,
            cost: self.cost,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a reservation metric in status planned.
pub fn new_reservation_metric(code: &str, label: &str) -> ReservationMetric {
    ReservationMetric {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        key_id: "".to_string(),
        amount: 0.0,
        cost: 0.0,
        ratio: 0.0,
        status: ReservationMetricStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_reservation_metric("code-233", "label-437");
        assert_eq!(item.status, ReservationMetricStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_reservation_metric("code-233", "label-437");
        if item.move_to(ReservationMetricStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationMetricStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_reservation_metric("code-233", "label-437");
        item.status = ReservationMetricStatus::Cancelled;
        assert!(
            item.move_to(ReservationMetricStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
