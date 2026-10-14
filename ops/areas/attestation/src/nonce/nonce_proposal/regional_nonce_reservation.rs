use super::*;

use acme_kit::text;

/// Where a regional nonce reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalNonceReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalNonceReservationStatus {
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

impl std::fmt::Display for RegionalNonceReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional nonce reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalNonceReservation {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub floor_value: f64,
    pub issuer: String,
    pub active: bool,
    pub pinned: bool,
    pub ttl_seconds: i64,
    pub status: RegionalNonceReservationStatus,
}

impl RegionalNonceReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalNonceReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalNonceReservationStatus) -> bool {
        match self.status {
            RegionalNonceReservationStatus::Draft => {
                match next {
                    RegionalNonceReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalNonceReservationStatus::Pending => {
                return next == RegionalNonceReservationStatus::Active
                    || next == RegionalNonceReservationStatus::Closed;
            }
            RegionalNonceReservationStatus::Active => {
                match next {
                    RegionalNonceReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalNonceReservationStatus::Suspended => {
                return next == RegionalNonceReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalNonceReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional nonce reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalNonceReservation {
        RegionalNonceReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            floor_value: self.floor_value,
            issuer: self.issuer.clone(),
            active: self.active,
            pinned: self.pinned,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional nonce reservation in status draft.
pub fn new_regional_nonce_reservation(code: &str, title: &str) -> RegionalNonceReservation {
    RegionalNonceReservation {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        floor_value: 0.0,
        issuer: "".to_string(),
        active: false,
        pinned: false,
        ttl_seconds: 0,
        status: RegionalNonceReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_nonce_reservation("code-707", "title-249");
        assert_eq!(item.status, RegionalNonceReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_regional_nonce_reservation("code-707", "title-249");
        if item.move_to(RegionalNonceReservationStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalNonceReservationStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_nonce_reservation("code-707", "title-249");
        item.status = RegionalNonceReservationStatus::Closed;
        assert!(
            item.move_to(RegionalNonceReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
