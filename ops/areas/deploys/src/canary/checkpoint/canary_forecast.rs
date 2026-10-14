use super::*;

use acme_kit::text;

/// Where a canary forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CanaryForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CanaryForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary forecast as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryForecast {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub key_id: String,
    pub notes: String,
    pub margin: f64,
    pub reviewer: String,
    pub status: CanaryForecastStatus,
}

impl CanaryForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryForecastStatus) -> bool {
        match self.status {
            CanaryForecastStatus::Proposed => {
                return next == CanaryForecastStatus::Approved || next == CanaryForecastStatus::Rejected;
            }
            CanaryForecastStatus::Approved => {
                match next {
                    CanaryForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryForecastStatus::Rejected => {
                match next {
                    CanaryForecastStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: CanaryForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.margin + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryForecast {
        CanaryForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            key_id: self.key_id.clone(),
            notes: self.notes.clone(),
            margin: self.margin,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a canary forecast in status proposed.
pub fn new_canary_forecast(code: &str, label: &str) -> CanaryForecast {
    CanaryForecast {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        key_id: "".to_string(),
        notes: "".to_string(),
        margin: 0.0,
        reviewer: "".to_string(),
        status: CanaryForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_canary_forecast("code-446", "label-812");
        assert_eq!(item.status, CanaryForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_canary_forecast("code-446", "label-812");
        if item.move_to(CanaryForecastStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryForecastStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_canary_forecast("code-446", "label-812");
        item.status = CanaryForecastStatus::Withdrawn;
        assert!(
            item.move_to(CanaryForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
