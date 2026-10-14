use super::*;

use acme_kit::text;

/// Where a legacy release forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyReleaseForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyReleaseForecastStatus {
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

impl std::fmt::Display for LegacyReleaseForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy release forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReleaseForecast {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub channel: String,
    pub minimum: f64,
    pub tenant_id: String,
    pub status: LegacyReleaseForecastStatus,
}

impl LegacyReleaseForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyReleaseForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyReleaseForecastStatus) -> bool {
        match self.status {
            LegacyReleaseForecastStatus::New => {
                return next == LegacyReleaseForecastStatus::Failed;
            }
            LegacyReleaseForecastStatus::Queued => {
                return next == LegacyReleaseForecastStatus::Done;
            }
            LegacyReleaseForecastStatus::Running => {
                match next {
                    LegacyReleaseForecastStatus::Done | LegacyReleaseForecastStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyReleaseForecastStatus::Done => {
                match next {
                    LegacyReleaseForecastStatus::Failed => {
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

    pub fn move_to(&mut self, next: LegacyReleaseForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy release forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> LegacyReleaseForecast {
        LegacyReleaseForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            channel: self.channel.clone(),
            minimum: self.minimum,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a legacy release forecast in status new.
pub fn new_legacy_release_forecast(code: &str, title: &str) -> LegacyReleaseForecast {
    LegacyReleaseForecast {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        channel: "".to_string(),
        minimum: 0.0,
        tenant_id: "".to_string(),
        status: LegacyReleaseForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_release_forecast("code-220", "title-688");
        assert_eq!(item.status, LegacyReleaseForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_legacy_release_forecast("code-220", "title-688");
        if item.move_to(LegacyReleaseForecastStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyReleaseForecastStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_release_forecast("code-220", "title-688");
        item.status = LegacyReleaseForecastStatus::Failed;
        assert!(
            item.move_to(LegacyReleaseForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
