use super::*;

use acme_kit::text;

/// Where a global reservation variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalReservationVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalReservationVarianceStatus {
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

impl std::fmt::Display for GlobalReservationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global reservation variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationVariance {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub created_at: i64,
    pub ttl_seconds: i64,
    pub starts_at: i64,
    pub version: i64,
    pub weight: f64,
    pub quantity: i64,
    pub amount: f64,
    pub locale: String,
    pub status: GlobalReservationVarianceStatus,
}

impl GlobalReservationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalReservationVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalReservationVarianceStatus) -> bool {
        match self.status {
            GlobalReservationVarianceStatus::Planned => {
                match next {
                    GlobalReservationVarianceStatus::Scheduled | GlobalReservationVarianceStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReservationVarianceStatus::Scheduled => {
                return next == GlobalReservationVarianceStatus::InProgress;
            }
            GlobalReservationVarianceStatus::InProgress => {
                match next {
                    GlobalReservationVarianceStatus::Completed | GlobalReservationVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReservationVarianceStatus::Completed => {
                match next {
                    GlobalReservationVarianceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: GlobalReservationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global reservation variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.quantity as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalReservationVariance {
        GlobalReservationVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            created_at: self.created_at,
            ttl_seconds: self.ttl_seconds,
            starts_at: self.starts_at,
            version: self.version,
            weight: self.weight,
            quantity: self.quantity,
            amount: self.amount,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a global reservation variance in status planned.
pub fn new_global_reservation_variance(code: &str, title: &str) -> GlobalReservationVariance {
    GlobalReservationVariance {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        created_at: 0,
        ttl_seconds: 0,
        starts_at: 0,
        version: 0,
        weight: 0.0,
        quantity: 0,
        amount: 0.0,
        locale: "".to_string(),
        status: GlobalReservationVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_reservation_variance("code-388", "title-360");
        assert_eq!(item.status, GlobalReservationVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_global_reservation_variance("code-388", "title-360");
        if item.move_to(GlobalReservationVarianceStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalReservationVarianceStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_reservation_variance("code-388", "title-360");
        item.status = GlobalReservationVarianceStatus::Cancelled;
        assert!(
            item.move_to(GlobalReservationVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
