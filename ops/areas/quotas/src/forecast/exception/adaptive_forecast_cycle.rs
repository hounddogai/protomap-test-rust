use super::*;

use acme_kit::text;

/// Where a adaptive forecast cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveForecastCycleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveForecastCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveForecastCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive forecast cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastCycle {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub discount: f64,
    pub sequence: i64,
    pub host: String,
    pub created_at: i64,
    pub notes: String,
    pub summary: String,
    pub status: AdaptiveForecastCycleStatus,
}

impl AdaptiveForecastCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveForecastCycleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveForecastCycleStatus) -> bool {
        match self.status {
            AdaptiveForecastCycleStatus::Requested => {
                return next == AdaptiveForecastCycleStatus::Revoked;
            }
            AdaptiveForecastCycleStatus::Granted => {
                return next == AdaptiveForecastCycleStatus::Revoked;
            }
            AdaptiveForecastCycleStatus::Revoked => {
                return next == AdaptiveForecastCycleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveForecastCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive forecast cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.sequence as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveForecastCycle {
        AdaptiveForecastCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            discount: self.discount,
            sequence: self.sequence,
            host: self.host.clone(),
            created_at: self.created_at,
            notes: self.notes.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive forecast cycle in status requested.
pub fn new_adaptive_forecast_cycle(code: &str, title: &str) -> AdaptiveForecastCycle {
    AdaptiveForecastCycle {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        discount: 0.0,
        sequence: 0,
        host: "".to_string(),
        created_at: 0,
        notes: "".to_string(),
        summary: "".to_string(),
        status: AdaptiveForecastCycleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_adaptive_forecast_cycle("code-962", "title-158");
        assert_eq!(item.status, AdaptiveForecastCycleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_adaptive_forecast_cycle("code-962", "title-158");
        if item.move_to(AdaptiveForecastCycleStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveForecastCycleStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_adaptive_forecast_cycle("code-962", "title-158");
        item.status = AdaptiveForecastCycleStatus::Expired;
        assert!(
            item.move_to(AdaptiveForecastCycleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
