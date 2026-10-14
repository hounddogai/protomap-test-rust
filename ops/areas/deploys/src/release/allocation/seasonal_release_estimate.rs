use super::*;

use acme_kit::text;

/// Where a seasonal release estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalReleaseEstimateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalReleaseEstimateStatus {
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

impl std::fmt::Display for SeasonalReleaseEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal release estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseEstimate {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub limit: i64,
    pub locale: String,
    pub title: String,
    pub status: SeasonalReleaseEstimateStatus,
}

impl SeasonalReleaseEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalReleaseEstimateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalReleaseEstimateStatus) -> bool {
        match self.status {
            SeasonalReleaseEstimateStatus::New => {
                match next {
                    SeasonalReleaseEstimateStatus::Queued | SeasonalReleaseEstimateStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalReleaseEstimateStatus::Queued => {
                return next == SeasonalReleaseEstimateStatus::Running || next == SeasonalReleaseEstimateStatus::Failed;
            }
            SeasonalReleaseEstimateStatus::Running => {
                match next {
                    SeasonalReleaseEstimateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalReleaseEstimateStatus::Done => {
                match next {
                    SeasonalReleaseEstimateStatus::Failed => {
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

    pub fn move_to(&mut self, next: SeasonalReleaseEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal release estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> SeasonalReleaseEstimate {
        SeasonalReleaseEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            limit: self.limit,
            locale: self.locale.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a seasonal release estimate in status new.
pub fn new_seasonal_release_estimate(code: &str, name: &str) -> SeasonalReleaseEstimate {
    SeasonalReleaseEstimate {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        limit: 0,
        locale: "".to_string(),
        title: "".to_string(),
        status: SeasonalReleaseEstimateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_release_estimate("code-331", "name-145");
        assert_eq!(item.status, SeasonalReleaseEstimateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_seasonal_release_estimate("code-331", "name-145");
        if item.move_to(SeasonalReleaseEstimateStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalReleaseEstimateStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_release_estimate("code-331", "name-145");
        item.status = SeasonalReleaseEstimateStatus::Failed;
        assert!(
            item.move_to(SeasonalReleaseEstimateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
