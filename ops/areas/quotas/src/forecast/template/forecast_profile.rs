use super::*;

use acme_kit::text;

/// Where a forecast profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ForecastProfileStatus {
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

impl std::fmt::Display for ForecastProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProfile {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub tags: Vec<String>,
    pub ceiling: f64,
    pub category: String,
    pub country: String,
    pub fingerprint: String,
    pub attempts: i64,
    pub notes: String,
    pub rotation_days: i64,
    pub status: ForecastProfileStatus,
}

impl ForecastProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastProfileStatus) -> bool {
        match self.status {
            ForecastProfileStatus::New => {
                match next {
                    ForecastProfileStatus::Queued | ForecastProfileStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastProfileStatus::Queued => {
                return next == ForecastProfileStatus::Done || next == ForecastProfileStatus::Failed;
            }
            ForecastProfileStatus::Running => {
                match next {
                    ForecastProfileStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastProfileStatus::Done => {
                return next == ForecastProfileStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast profile {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "forecast profile", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.rotation_days as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastProfile {
        ForecastProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            tags: self.tags.clone(),
            ceiling: self.ceiling,
            category: self.category.clone(),
            country: self.country.clone(),
            fingerprint: self.fingerprint.clone(),
            attempts: self.attempts,
            notes: self.notes.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a forecast profile in status new.
pub fn new_forecast_profile(code: &str, name: &str) -> ForecastProfile {
    ForecastProfile {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        tags: Vec::new(),
        ceiling: 0.0,
        category: "".to_string(),
        country: "".to_string(),
        fingerprint: "".to_string(),
        attempts: 0,
        notes: "".to_string(),
        rotation_days: 0,
        status: ForecastProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_forecast_profile("code-904", "name-753");
        assert_eq!(item.status, ForecastProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_forecast_profile("code-904", "name-753");
        if item.move_to(ForecastProfileStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastProfileStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_forecast_profile("code-904", "name-753");
        item.status = ForecastProfileStatus::Failed;
        assert!(
            item.move_to(ForecastProfileStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
