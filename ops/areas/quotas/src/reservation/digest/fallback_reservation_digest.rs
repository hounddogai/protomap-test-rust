use super::*;

use acme_kit::text;

/// Where a fallback reservation digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackReservationDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackReservationDigestStatus {
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

impl std::fmt::Display for FallbackReservationDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback reservation digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationDigest {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub locale: String,
    pub score: f64,
    pub country: String,
    pub source: String,
    pub margin: f64,
    pub burst: i64,
    pub ends_at: i64,
    pub status: FallbackReservationDigestStatus,
}

impl FallbackReservationDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackReservationDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackReservationDigestStatus) -> bool {
        match self.status {
            FallbackReservationDigestStatus::Planned => {
                match next {
                    FallbackReservationDigestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReservationDigestStatus::Scheduled => {
                match next {
                    FallbackReservationDigestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReservationDigestStatus::InProgress => {
                match next {
                    FallbackReservationDigestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReservationDigestStatus::Completed => {
                return next == FallbackReservationDigestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackReservationDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback reservation digest {} cannot move from {} to {}",
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
        format!("{} {}", "fallback reservation digest", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.burst as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackReservationDigest {
        FallbackReservationDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            locale: self.locale.clone(),
            score: self.score,
            country: self.country.clone(),
            source: self.source.clone(),
            margin: self.margin,
            burst: self.burst,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a fallback reservation digest in status planned.
pub fn new_fallback_reservation_digest(code: &str, name: &str) -> FallbackReservationDigest {
    FallbackReservationDigest {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        locale: "".to_string(),
        score: 0.0,
        country: "".to_string(),
        source: "".to_string(),
        margin: 0.0,
        burst: 0,
        ends_at: 0,
        status: FallbackReservationDigestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_reservation_digest("code-172", "name-448");
        assert_eq!(item.status, FallbackReservationDigestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_fallback_reservation_digest("code-172", "name-448");
        if item.move_to(FallbackReservationDigestStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackReservationDigestStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_reservation_digest("code-172", "name-448");
        item.status = FallbackReservationDigestStatus::Cancelled;
        assert!(
            item.move_to(FallbackReservationDigestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
