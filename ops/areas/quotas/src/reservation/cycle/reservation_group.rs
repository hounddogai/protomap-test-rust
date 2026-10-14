use super::*;

use acme_kit::text;

/// Where a reservation group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationGroupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ReservationGroupStatus {
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

impl std::fmt::Display for ReservationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationGroup {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub maximum: f64,
    pub port: i64,
    pub phone: String,
    pub updated_at: i64,
    pub source: String,
    pub status: ReservationGroupStatus,
}

impl ReservationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationGroupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationGroupStatus) -> bool {
        match self.status {
            ReservationGroupStatus::Draft => {
                return next == ReservationGroupStatus::Pending;
            }
            ReservationGroupStatus::Pending => {
                return next == ReservationGroupStatus::Active || next == ReservationGroupStatus::Closed;
            }
            ReservationGroupStatus::Active => {
                return next == ReservationGroupStatus::Suspended || next == ReservationGroupStatus::Closed;
            }
            ReservationGroupStatus::Suspended => {
                match next {
                    ReservationGroupStatus::Closed => {
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

    pub fn move_to(&mut self, next: ReservationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation group {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.burst as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationGroup {
        ReservationGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            maximum: self.maximum,
            port: self.port,
            phone: self.phone.clone(),
            updated_at: self.updated_at,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a reservation group in status draft.
pub fn new_reservation_group(code: &str, label: &str) -> ReservationGroup {
    ReservationGroup {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        maximum: 0.0,
        port: 0,
        phone: "".to_string(),
        updated_at: 0,
        source: "".to_string(),
        status: ReservationGroupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_reservation_group("code-407", "label-297");
        assert_eq!(item.status, ReservationGroupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_reservation_group("code-407", "label-297");
        if item.move_to(ReservationGroupStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationGroupStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_reservation_group("code-407", "label-297");
        item.status = ReservationGroupStatus::Closed;
        assert!(
            item.move_to(ReservationGroupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
