use super::*;

use acme_kit::text;

/// Where a rollout variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RolloutVarianceStatus {
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

impl std::fmt::Display for RolloutVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout variance as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutVariance {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub audience: String,
    pub retries: i64,
    pub summary: String,
    pub expires_at: i64,
    pub burst: i64,
    pub ttl_seconds: i64,
    pub status: RolloutVarianceStatus,
}

impl RolloutVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutVarianceStatus) -> bool {
        match self.status {
            RolloutVarianceStatus::New => {
                match next {
                    RolloutVarianceStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutVarianceStatus::Queued => {
                return next == RolloutVarianceStatus::Done || next == RolloutVarianceStatus::Failed;
            }
            RolloutVarianceStatus::Running => {
                match next {
                    RolloutVarianceStatus::Done | RolloutVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutVarianceStatus::Done => {
                return next == RolloutVarianceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollout variance", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.burst as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutVariance {
        RolloutVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            audience: self.audience.clone(),
            retries: self.retries,
            summary: self.summary.clone(),
            expires_at: self.expires_at,
            burst: self.burst,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rollout variance in status new.
pub fn new_rollout_variance(code: &str, title: &str) -> RolloutVariance {
    RolloutVariance {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        audience: "".to_string(),
        retries: 0,
        summary: "".to_string(),
        expires_at: 0,
        burst: 0,
        ttl_seconds: 0,
        status: RolloutVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rollout_variance("code-425", "title-969");
        assert_eq!(item.status, RolloutVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rollout_variance("code-425", "title-969");
        if item.move_to(RolloutVarianceStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutVarianceStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rollout_variance("code-425", "title-969");
        item.status = RolloutVarianceStatus::Failed;
        assert!(
            item.move_to(RolloutVarianceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
