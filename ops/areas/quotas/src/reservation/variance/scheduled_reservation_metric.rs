use super::*;

use acme_kit::text;

/// Where a scheduled reservation metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledReservationMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledReservationMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ScheduledReservationMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled reservation metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationMetric {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub timezone: String,
    pub margin: f64,
    pub status: ScheduledReservationMetricStatus,
}

impl ScheduledReservationMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledReservationMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledReservationMetricStatus) -> bool {
        match self.status {
            ScheduledReservationMetricStatus::Draft => {
                match next {
                    ScheduledReservationMetricStatus::Pending | ScheduledReservationMetricStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledReservationMetricStatus::Pending => {
                match next {
                    ScheduledReservationMetricStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledReservationMetricStatus::Active => {
                match next {
                    ScheduledReservationMetricStatus::Suspended | ScheduledReservationMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledReservationMetricStatus::Suspended => {
                match next {
                    ScheduledReservationMetricStatus::Closed => {
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

    pub fn move_to(&mut self, next: ScheduledReservationMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled reservation metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled reservation metric", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> ScheduledReservationMetric {
        ScheduledReservationMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            timezone: self.timezone.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled reservation metric in status draft.
pub fn new_scheduled_reservation_metric(code: &str, label: &str) -> ScheduledReservationMetric {
    ScheduledReservationMetric {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        timezone: "".to_string(),
        margin: 0.0,
        status: ScheduledReservationMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_reservation_metric("code-685", "label-687");
        assert_eq!(item.status, ScheduledReservationMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_scheduled_reservation_metric("code-685", "label-687");
        if item.move_to(ScheduledReservationMetricStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledReservationMetricStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_reservation_metric("code-685", "label-687");
        item.status = ScheduledReservationMetricStatus::Closed;
        assert!(
            item.move_to(ScheduledReservationMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
