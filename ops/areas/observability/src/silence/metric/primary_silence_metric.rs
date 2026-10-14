use super::*;

use acme_kit::text;

/// Where a primary silence metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySilenceMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimarySilenceMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PrimarySilenceMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary silence metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceMetric {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub external_ref: String,
    pub channel: String,
    pub phone: String,
    pub locale: String,
    pub priority: i64,
    pub status: PrimarySilenceMetricStatus,
}

impl PrimarySilenceMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySilenceMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySilenceMetricStatus) -> bool {
        match self.status {
            PrimarySilenceMetricStatus::New => {
                return next == PrimarySilenceMetricStatus::Queued || next == PrimarySilenceMetricStatus::Failed;
            }
            PrimarySilenceMetricStatus::Queued => {
                match next {
                    PrimarySilenceMetricStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySilenceMetricStatus::Running => {
                return next == PrimarySilenceMetricStatus::Done;
            }
            PrimarySilenceMetricStatus::Done => {
                match next {
                    PrimarySilenceMetricStatus::Failed => {
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

    pub fn move_to(&mut self, next: PrimarySilenceMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary silence metric {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.priority as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySilenceMetric {
        PrimarySilenceMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            external_ref: self.external_ref.clone(),
            channel: self.channel.clone(),
            phone: self.phone.clone(),
            locale: self.locale.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a primary silence metric in status new.
pub fn new_primary_silence_metric(code: &str, label: &str) -> PrimarySilenceMetric {
    PrimarySilenceMetric {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        external_ref: "".to_string(),
        channel: "".to_string(),
        phone: "".to_string(),
        locale: "".to_string(),
        priority: 0,
        status: PrimarySilenceMetricStatus::New,
    }
}
