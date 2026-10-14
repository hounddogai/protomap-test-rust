use super::*;

use acme_kit::text;

/// Where a primary reservation summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryReservationSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryReservationSummaryStatus {
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

impl std::fmt::Display for PrimaryReservationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary reservation summary as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationSummary {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub capacity: i64,
    pub audience: String,
    pub status: PrimaryReservationSummaryStatus,
}

impl PrimaryReservationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryReservationSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryReservationSummaryStatus) -> bool {
        match self.status {
            PrimaryReservationSummaryStatus::Draft => {
                match next {
                    PrimaryReservationSummaryStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryReservationSummaryStatus::Pending => {
                match next {
                    PrimaryReservationSummaryStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryReservationSummaryStatus::Active => {
                return next == PrimaryReservationSummaryStatus::Closed;
            }
            PrimaryReservationSummaryStatus::Suspended => {
                match next {
                    PrimaryReservationSummaryStatus::Closed => {
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

    pub fn move_to(&mut self, next: PrimaryReservationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary reservation summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary reservation summary", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryReservationSummary {
        PrimaryReservationSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            capacity: self.capacity,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary reservation summary in status draft.
pub fn new_primary_reservation_summary(code: &str, label: &str) -> PrimaryReservationSummary {
    PrimaryReservationSummary {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        capacity: 0,
        audience: "".to_string(),
        status: PrimaryReservationSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_reservation_summary("code-88", "label-63");
        assert_eq!(item.status, PrimaryReservationSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_primary_reservation_summary("code-88", "label-63");
        if item.move_to(PrimaryReservationSummaryStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryReservationSummaryStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_reservation_summary("code-88", "label-63");
        item.status = PrimaryReservationSummaryStatus::Closed;
        assert!(
            item.move_to(PrimaryReservationSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
