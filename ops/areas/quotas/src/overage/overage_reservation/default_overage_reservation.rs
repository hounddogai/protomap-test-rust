use super::*;

use acme_kit::text;

/// Where a default overage reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultOverageReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultOverageReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultOverageReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default overage reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReservation {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub currency: String,
    pub timezone: String,
    pub host: String,
    pub ratio: f64,
    pub summary: String,
    pub rate: f64,
    pub issuer: String,
    pub status: DefaultOverageReservationStatus,
}

impl DefaultOverageReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultOverageReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultOverageReservationStatus) -> bool {
        match self.status {
            DefaultOverageReservationStatus::Requested => {
                match next {
                    DefaultOverageReservationStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultOverageReservationStatus::Granted => {
                match next {
                    DefaultOverageReservationStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultOverageReservationStatus::Revoked => {
                match next {
                    DefaultOverageReservationStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultOverageReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default overage reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        (self.ratio * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultOverageReservation {
        DefaultOverageReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            currency: self.currency.clone(),
            timezone: self.timezone.clone(),
            host: self.host.clone(),
            ratio: self.ratio,
            summary: self.summary.clone(),
            rate: self.rate,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default overage reservation in status requested.
pub fn new_default_overage_reservation(code: &str, title: &str) -> DefaultOverageReservation {
    DefaultOverageReservation {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        currency: "".to_string(),
        timezone: "".to_string(),
        host: "".to_string(),
        ratio: 0.0,
        summary: "".to_string(),
        rate: 0.0,
        issuer: "".to_string(),
        status: DefaultOverageReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_overage_reservation("code-964", "title-83");
        assert_eq!(item.status, DefaultOverageReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_default_overage_reservation("code-964", "title-83");
        if item.move_to(DefaultOverageReservationStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultOverageReservationStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_overage_reservation("code-964", "title-83");
        item.status = DefaultOverageReservationStatus::Expired;
        assert!(
            item.move_to(DefaultOverageReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
