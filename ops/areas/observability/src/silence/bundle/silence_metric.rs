use super::*;

use acme_kit::text;

/// Where a silence metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceMetricStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SilenceMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SilenceMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceMetric {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub summary: String,
    pub issuer: String,
    pub principal: String,
    pub unit: String,
    pub quota_units: i64,
    pub ends_at: i64,
    pub status: SilenceMetricStatus,
}

impl SilenceMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceMetricStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceMetricStatus) -> bool {
        match self.status {
            SilenceMetricStatus::Requested => {
                match next {
                    SilenceMetricStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceMetricStatus::Granted => {
                return next == SilenceMetricStatus::Revoked || next == SilenceMetricStatus::Expired;
            }
            SilenceMetricStatus::Revoked => {
                match next {
                    SilenceMetricStatus::Expired => {
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

    pub fn move_to(&mut self, next: SilenceMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence metric {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "silence metric", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quota_units as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceMetric {
        SilenceMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            summary: self.summary.clone(),
            issuer: self.issuer.clone(),
            principal: self.principal.clone(),
            unit: self.unit.clone(),
            quota_units: self.quota_units,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a silence metric in status requested.
pub fn new_silence_metric(code: &str, label: &str) -> SilenceMetric {
    SilenceMetric {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        summary: "".to_string(),
        issuer: "".to_string(),
        principal: "".to_string(),
        unit: "".to_string(),
        quota_units: 0,
        ends_at: 0,
        status: SilenceMetricStatus::Requested,
    }
}
