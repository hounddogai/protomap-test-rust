use super::*;

use acme_kit::text;

/// Where a batched token forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTokenForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedTokenForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedTokenForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched token forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenForecast {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub updated_at: i64,
    pub issuer: String,
    pub source: String,
    pub port: i64,
    pub ttl_seconds: i64,
    pub segment: String,
    pub status: BatchedTokenForecastStatus,
}

impl BatchedTokenForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTokenForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTokenForecastStatus) -> bool {
        match self.status {
            BatchedTokenForecastStatus::Requested => {
                return next == BatchedTokenForecastStatus::Revoked;
            }
            BatchedTokenForecastStatus::Granted => {
                return next == BatchedTokenForecastStatus::Revoked || next == BatchedTokenForecastStatus::Expired;
            }
            BatchedTokenForecastStatus::Revoked => {
                return next == BatchedTokenForecastStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedTokenForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched token forecast {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedTokenForecast {
        BatchedTokenForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            updated_at: self.updated_at,
            issuer: self.issuer.clone(),
            source: self.source.clone(),
            port: self.port,
            ttl_seconds: self.ttl_seconds,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a batched token forecast in status requested.
pub fn new_batched_token_forecast(code: &str, label: &str) -> BatchedTokenForecast {
    BatchedTokenForecast {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        updated_at: 0,
        issuer: "".to_string(),
        source: "".to_string(),
        port: 0,
        ttl_seconds: 0,
        segment: "".to_string(),
        status: BatchedTokenForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_token_forecast("code-432", "label-466");
        assert_eq!(item.status, BatchedTokenForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_batched_token_forecast("code-432", "label-466");
        if item.move_to(BatchedTokenForecastStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedTokenForecastStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_token_forecast("code-432", "label-466");
        item.status = BatchedTokenForecastStatus::Expired;
        assert!(
            item.move_to(BatchedTokenForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
