use super::*;

use acme_kit::text;

/// Where a primary forecast score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryForecastScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryForecastScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryForecastScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary forecast score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastScore {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub cluster: String,
    pub score: f64,
    pub description: String,
    pub starts_at: i64,
    pub issuer: String,
    pub title: String,
    pub attempts: i64,
    pub status: PrimaryForecastScoreStatus,
}

impl PrimaryForecastScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryForecastScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryForecastScoreStatus) -> bool {
        match self.status {
            PrimaryForecastScoreStatus::Requested => {
                return next == PrimaryForecastScoreStatus::Revoked || next == PrimaryForecastScoreStatus::Expired;
            }
            PrimaryForecastScoreStatus::Granted => {
                match next {
                    PrimaryForecastScoreStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryForecastScoreStatus::Revoked => {
                match next {
                    PrimaryForecastScoreStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryForecastScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary forecast score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.version as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryForecastScore {
        PrimaryForecastScore {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            cluster: self.cluster.clone(),
            score: self.score,
            description: self.description.clone(),
            starts_at: self.starts_at,
            issuer: self.issuer.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary forecast score in status requested.
pub fn new_primary_forecast_score(code: &str, label: &str) -> PrimaryForecastScore {
    PrimaryForecastScore {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        cluster: "".to_string(),
        score: 0.0,
        description: "".to_string(),
        starts_at: 0,
        issuer: "".to_string(),
        title: "".to_string(),
        attempts: 0,
        status: PrimaryForecastScoreStatus::Requested,
    }
}
