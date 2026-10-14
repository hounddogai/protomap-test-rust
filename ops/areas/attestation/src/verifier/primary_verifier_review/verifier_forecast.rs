use super::*;

use acme_kit::text;

/// Where a verifier forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl VerifierForecastStatus {
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

impl std::fmt::Display for VerifierForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a verifier forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierForecast {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub cluster: String,
    pub limit: i64,
    pub batch_size: i64,
    pub channel: String,
    pub created_at: i64,
    pub status: VerifierForecastStatus,
}

impl VerifierForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierForecastStatus) -> bool {
        match self.status {
            VerifierForecastStatus::Planned => {
                return next == VerifierForecastStatus::Scheduled;
            }
            VerifierForecastStatus::Scheduled => {
                return next == VerifierForecastStatus::InProgress;
            }
            VerifierForecastStatus::InProgress => {
                match next {
                    VerifierForecastStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierForecastStatus::Completed => {
                match next {
                    VerifierForecastStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: VerifierForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier forecast {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.limit as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierForecast {
        VerifierForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            cluster: self.cluster.clone(),
            limit: self.limit,
            batch_size: self.batch_size,
            channel: self.channel.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a verifier forecast in status planned.
pub fn new_verifier_forecast(code: &str, label: &str) -> VerifierForecast {
    VerifierForecast {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        cluster: "".to_string(),
        limit: 0,
        batch_size: 0,
        channel: "".to_string(),
        created_at: 0,
        status: VerifierForecastStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_verifier_forecast("code-868", "label-248");
        assert_eq!(item.status, VerifierForecastStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_verifier_forecast("code-868", "label-248");
        if item.move_to(VerifierForecastStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierForecastStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_verifier_forecast("code-868", "label-248");
        item.status = VerifierForecastStatus::Cancelled;
        assert!(
            item.move_to(VerifierForecastStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
