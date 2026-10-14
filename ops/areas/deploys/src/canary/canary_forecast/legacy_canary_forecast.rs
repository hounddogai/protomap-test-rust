use super::*;

use acme_kit::text;

/// Where a legacy canary forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyCanaryForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyCanaryForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyCanaryForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy canary forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryForecast {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub quantity: i64,
    pub port: i64,
    pub cipher: String,
    pub status: LegacyCanaryForecastStatus,
}

impl LegacyCanaryForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyCanaryForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyCanaryForecastStatus) -> bool {
        match self.status {
            LegacyCanaryForecastStatus::Proposed => {
                return next == LegacyCanaryForecastStatus::Approved || next == LegacyCanaryForecastStatus::Rejected;
            }
            LegacyCanaryForecastStatus::Approved => {
                match next {
                    LegacyCanaryForecastStatus::Rejected | LegacyCanaryForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyCanaryForecastStatus::Rejected => {
                return next == LegacyCanaryForecastStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyCanaryForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy canary forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy canary forecast", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.port as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyCanaryForecast {
        LegacyCanaryForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            quantity: self.quantity,
            port: self.port,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy canary forecast in status proposed.
pub fn new_legacy_canary_forecast(code: &str, title: &str) -> LegacyCanaryForecast {
    LegacyCanaryForecast {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        quantity: 0,
        port: 0,
        cipher: "".to_string(),
        status: LegacyCanaryForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_legacy_canary_forecast("code-649", "title-585");
        assert_eq!(item.status, LegacyCanaryForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_legacy_canary_forecast("code-649", "title-585");
        if item.move_to(LegacyCanaryForecastStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyCanaryForecastStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_legacy_canary_forecast("code-649", "title-585");
        item.status = LegacyCanaryForecastStatus::Withdrawn;
        assert!(
            item.move_to(LegacyCanaryForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
