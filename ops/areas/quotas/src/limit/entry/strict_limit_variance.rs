use super::*;

use acme_kit::text;

/// Where a strict limit variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictLimitVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictLimitVarianceStatus {
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

impl std::fmt::Display for StrictLimitVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict limit variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictLimitVariance {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub name: String,
    pub window_days: i64,
    pub notes: String,
    pub timezone: String,
    pub owner_id: String,
    pub batch_size: i64,
    pub status: StrictLimitVarianceStatus,
}

impl StrictLimitVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictLimitVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictLimitVarianceStatus) -> bool {
        match self.status {
            StrictLimitVarianceStatus::New => {
                match next {
                    StrictLimitVarianceStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictLimitVarianceStatus::Queued => {
                match next {
                    StrictLimitVarianceStatus::Done | StrictLimitVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictLimitVarianceStatus::Running => {
                match next {
                    StrictLimitVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictLimitVarianceStatus::Done => {
                match next {
                    StrictLimitVarianceStatus::Failed => {
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

    pub fn move_to(&mut self, next: StrictLimitVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict limit variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.rotation_days as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictLimitVariance {
        StrictLimitVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            name: self.name.clone(),
            window_days: self.window_days,
            notes: self.notes.clone(),
            timezone: self.timezone.clone(),
            owner_id: self.owner_id.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict limit variance in status new.
pub fn new_strict_limit_variance(code: &str, title: &str) -> StrictLimitVariance {
    StrictLimitVariance {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        name: "".to_string(),
        window_days: 0,
        notes: "".to_string(),
        timezone: "".to_string(),
        owner_id: "".to_string(),
        batch_size: 0,
        status: StrictLimitVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_limit_variance("code-136", "title-999");
        assert_eq!(item.status, StrictLimitVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_strict_limit_variance("code-136", "title-999");
        if item.move_to(StrictLimitVarianceStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictLimitVarianceStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_limit_variance("code-136", "title-999");
        item.status = StrictLimitVarianceStatus::Failed;
        assert!(
            item.move_to(StrictLimitVarianceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
