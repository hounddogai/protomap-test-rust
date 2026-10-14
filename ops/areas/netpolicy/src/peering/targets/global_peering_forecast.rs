use super::*;

use acme_kit::text;

/// Where a global peering forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPeeringForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalPeeringForecastStatus {
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

impl std::fmt::Display for GlobalPeeringForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global peering forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringForecast {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub currency: String,
    pub issuer: String,
    pub duration_minutes: i64,
    pub tags: Vec<String>,
    pub status: GlobalPeeringForecastStatus,
}

impl GlobalPeeringForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPeeringForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPeeringForecastStatus) -> bool {
        match self.status {
            GlobalPeeringForecastStatus::New => {
                match next {
                    GlobalPeeringForecastStatus::Done | GlobalPeeringForecastStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPeeringForecastStatus::Queued => {
                return next == GlobalPeeringForecastStatus::Running;
            }
            GlobalPeeringForecastStatus::Running => {
                match next {
                    GlobalPeeringForecastStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPeeringForecastStatus::Done => {
                match next {
                    GlobalPeeringForecastStatus::Failed => {
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

    pub fn move_to(&mut self, next: GlobalPeeringForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global peering forecast {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPeeringForecast {
        GlobalPeeringForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            currency: self.currency.clone(),
            issuer: self.issuer.clone(),
            duration_minutes: self.duration_minutes,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global peering forecast in status new.
pub fn new_global_peering_forecast(code: &str, title: &str) -> GlobalPeeringForecast {
    GlobalPeeringForecast {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        currency: "".to_string(),
        issuer: "".to_string(),
        duration_minutes: 0,
        tags: Vec::new(),
        status: GlobalPeeringForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_peering_forecast("code-263", "title-829");
        assert_eq!(item.status, GlobalPeeringForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_global_peering_forecast("code-263", "title-829");
        if item.move_to(GlobalPeeringForecastStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPeeringForecastStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_peering_forecast("code-263", "title-829");
        item.status = GlobalPeeringForecastStatus::Failed;
        assert!(
            item.move_to(GlobalPeeringForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
