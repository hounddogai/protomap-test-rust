use super::*;

use acme_kit::text;

/// Where a zone score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ZoneScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ZoneScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a zone score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneScore {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub duration_minutes: i64,
    pub ttl_seconds: i64,
    pub retries: i64,
    pub capacity: i64,
    pub amount: f64,
    pub country: String,
    pub maximum: f64,
    pub status: ZoneScoreStatus,
}

impl ZoneScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneScoreStatus) -> bool {
        match self.status {
            ZoneScoreStatus::Requested => {
                return next == ZoneScoreStatus::Expired;
            }
            ZoneScoreStatus::Granted => {
                match next {
                    ZoneScoreStatus::Revoked | ZoneScoreStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneScoreStatus::Revoked => {
                match next {
                    ZoneScoreStatus::Expired => {
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

    pub fn move_to(&mut self, next: ZoneScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.maximum * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneScore {
        ZoneScore {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            duration_minutes: self.duration_minutes,
            ttl_seconds: self.ttl_seconds,
            retries: self.retries,
            capacity: self.capacity,
            amount: self.amount,
            country: self.country.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a zone score in status requested.
pub fn new_zone_score(code: &str, name: &str) -> ZoneScore {
    ZoneScore {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        duration_minutes: 0,
        ttl_seconds: 0,
        retries: 0,
        capacity: 0,
        amount: 0.0,
        country: "".to_string(),
        maximum: 0.0,
        status: ZoneScoreStatus::Requested,
    }
}
