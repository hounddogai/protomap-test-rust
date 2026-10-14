use super::*;

use acme_kit::text;

/// Where a manual identity forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualIdentityForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualIdentityForecastStatus {
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

impl std::fmt::Display for ManualIdentityForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual identity forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityForecast {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub score: f64,
    pub tags: Vec<String>,
    pub maximum: f64,
    pub issuer: String,
    pub revision: i64,
    pub floor_value: f64,
    pub owner_id: String,
    pub status: ManualIdentityForecastStatus,
}

impl ManualIdentityForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualIdentityForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualIdentityForecastStatus) -> bool {
        match self.status {
            ManualIdentityForecastStatus::New => {
                return next == ManualIdentityForecastStatus::Failed;
            }
            ManualIdentityForecastStatus::Queued => {
                match next {
                    ManualIdentityForecastStatus::Done | ManualIdentityForecastStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualIdentityForecastStatus::Running => {
                return next == ManualIdentityForecastStatus::Done;
            }
            ManualIdentityForecastStatus::Done => {
                match next {
                    ManualIdentityForecastStatus::Failed => {
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

    pub fn move_to(&mut self, next: ManualIdentityForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual identity forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.revision as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualIdentityForecast {
        ManualIdentityForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            score: self.score,
            tags: self.tags.clone(),
            maximum: self.maximum,
            issuer: self.issuer.clone(),
            revision: self.revision,
            floor_value: self.floor_value,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual identity forecast in status new.
pub fn new_manual_identity_forecast(code: &str, title: &str) -> ManualIdentityForecast {
    ManualIdentityForecast {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        score: 0.0,
        tags: Vec::new(),
        maximum: 0.0,
        issuer: "".to_string(),
        revision: 0,
        floor_value: 0.0,
        owner_id: "".to_string(),
        status: ManualIdentityForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_identity_forecast("code-845", "title-770");
        assert_eq!(item.status, ManualIdentityForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_manual_identity_forecast("code-845", "title-770");
        if item.move_to(ManualIdentityForecastStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualIdentityForecastStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_identity_forecast("code-845", "title-770");
        item.status = ManualIdentityForecastStatus::Failed;
        assert!(
            item.move_to(ManualIdentityForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
