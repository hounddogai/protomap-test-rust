use super::*;

use acme_kit::text;

/// Where a seasonal egress batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalEgressBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalEgressBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalEgressBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal egress batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressBatch {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub currency: String,
    pub sequence: i64,
    pub expires_at: i64,
    pub audience: String,
    pub burst: i64,
    pub locale: String,
    pub floor_value: f64,
    pub fingerprint: String,
    pub status: SeasonalEgressBatchStatus,
}

impl SeasonalEgressBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalEgressBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalEgressBatchStatus) -> bool {
        match self.status {
            SeasonalEgressBatchStatus::Requested => {
                match next {
                    SeasonalEgressBatchStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalEgressBatchStatus::Granted => {
                match next {
                    SeasonalEgressBatchStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalEgressBatchStatus::Revoked => {
                match next {
                    SeasonalEgressBatchStatus::Expired => {
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

    pub fn move_to(&mut self, next: SeasonalEgressBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal egress batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.burst as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalEgressBatch {
        SeasonalEgressBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            currency: self.currency.clone(),
            sequence: self.sequence,
            expires_at: self.expires_at,
            audience: self.audience.clone(),
            burst: self.burst,
            locale: self.locale.clone(),
            floor_value: self.floor_value,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a seasonal egress batch in status requested.
pub fn new_seasonal_egress_batch(code: &str, title: &str) -> SeasonalEgressBatch {
    SeasonalEgressBatch {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        currency: "".to_string(),
        sequence: 0,
        expires_at: 0,
        audience: "".to_string(),
        burst: 0,
        locale: "".to_string(),
        floor_value: 0.0,
        fingerprint: "".to_string(),
        status: SeasonalEgressBatchStatus::Requested,
    }
}
