use super::*;

use acme_kit::text;

/// Where a evidence forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EvidenceForecastStatus {
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

impl std::fmt::Display for EvidenceForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a evidence forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceForecast {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub score: f64,
    pub namespace: String,
    pub visible: bool,
    pub starts_at: i64,
    pub status: EvidenceForecastStatus,
}

impl EvidenceForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceForecastStatus) -> bool {
        match self.status {
            EvidenceForecastStatus::Draft => {
                return next == EvidenceForecastStatus::Closed;
            }
            EvidenceForecastStatus::Pending => {
                match next {
                    EvidenceForecastStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceForecastStatus::Active => {
                match next {
                    EvidenceForecastStatus::Suspended | EvidenceForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceForecastStatus::Suspended => {
                return next == EvidenceForecastStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.score - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceForecast {
        EvidenceForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            score: self.score,
            namespace: self.namespace.clone(),
            visible: self.visible,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a evidence forecast in status draft.
pub fn new_evidence_forecast(code: &str, label: &str) -> EvidenceForecast {
    EvidenceForecast {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        score: 0.0,
        namespace: "".to_string(),
        visible: false,
        starts_at: 0,
        status: EvidenceForecastStatus::Draft,
    }
}
