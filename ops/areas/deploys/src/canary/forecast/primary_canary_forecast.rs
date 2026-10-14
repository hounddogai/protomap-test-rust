use super::*;

use acme_kit::text;

/// Where a primary canary forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryCanaryForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryCanaryForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PrimaryCanaryForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary canary forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryForecast {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub checksum: String,
    pub owner_id: String,
    pub starts_at: i64,
    pub ceiling: f64,
    pub status: PrimaryCanaryForecastStatus,
}

impl PrimaryCanaryForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryCanaryForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryCanaryForecastStatus) -> bool {
        match self.status {
            PrimaryCanaryForecastStatus::Open => {
                match next {
                    PrimaryCanaryForecastStatus::InReview | PrimaryCanaryForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryCanaryForecastStatus::InReview => {
                match next {
                    PrimaryCanaryForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryCanaryForecastStatus::Resolved => {
                match next {
                    PrimaryCanaryForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryCanaryForecastStatus::Reopened => {
                match next {
                    PrimaryCanaryForecastStatus::Archived => {
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

    pub fn move_to(&mut self, next: PrimaryCanaryForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary canary forecast {} cannot move from {} to {}",
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
        format!("{} {}", "primary canary forecast", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        (self.ceiling * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryCanaryForecast {
        PrimaryCanaryForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            checksum: self.checksum.clone(),
            owner_id: self.owner_id.clone(),
            starts_at: self.starts_at,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary canary forecast in status open.
pub fn new_primary_canary_forecast(code: &str, label: &str) -> PrimaryCanaryForecast {
    PrimaryCanaryForecast {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        checksum: "".to_string(),
        owner_id: "".to_string(),
        starts_at: 0,
        ceiling: 0.0,
        status: PrimaryCanaryForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_canary_forecast("code-411", "label-456");
        assert_eq!(item.status, PrimaryCanaryForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_primary_canary_forecast("code-411", "label-456");
        if item.move_to(PrimaryCanaryForecastStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryCanaryForecastStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_canary_forecast("code-411", "label-456");
        item.status = PrimaryCanaryForecastStatus::Archived;
        assert!(
            item.move_to(PrimaryCanaryForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
