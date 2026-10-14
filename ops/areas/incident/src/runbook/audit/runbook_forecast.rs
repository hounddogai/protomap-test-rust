use super::*;

use acme_kit::text;

/// Where a runbook forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RunbookForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RunbookForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a runbook forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookForecast {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub rotation_days: i64,
    pub namespace: String,
    pub threshold: f64,
    pub source: String,
    pub archived: bool,
    pub status: RunbookForecastStatus,
}

impl RunbookForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookForecastStatus) -> bool {
        match self.status {
            RunbookForecastStatus::Requested => {
                return next == RunbookForecastStatus::Expired;
            }
            RunbookForecastStatus::Granted => {
                match next {
                    RunbookForecastStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookForecastStatus::Revoked => {
                return next == RunbookForecastStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.threshold + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookForecast {
        RunbookForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            rotation_days: self.rotation_days,
            namespace: self.namespace.clone(),
            threshold: self.threshold,
            source: self.source.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a runbook forecast in status requested.
pub fn new_runbook_forecast(code: &str, label: &str) -> RunbookForecast {
    RunbookForecast {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        rotation_days: 0,
        namespace: "".to_string(),
        threshold: 0.0,
        source: "".to_string(),
        archived: false,
        status: RunbookForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_runbook_forecast("code-874", "label-701");
        assert_eq!(item.status, RunbookForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_runbook_forecast("code-874", "label-701");
        if item.move_to(RunbookForecastStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookForecastStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_runbook_forecast("code-874", "label-701");
        item.status = RunbookForecastStatus::Expired;
        assert!(
            item.move_to(RunbookForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
