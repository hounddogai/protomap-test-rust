use super::*;

use acme_kit::text;

/// Where a strict token plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictTokenPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictTokenPlanStatus {
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

impl std::fmt::Display for StrictTokenPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict token plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTokenPlan {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub port: i64,
    pub timezone: String,
    pub rate: f64,
    pub host: String,
    pub status: StrictTokenPlanStatus,
}

impl StrictTokenPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictTokenPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictTokenPlanStatus) -> bool {
        match self.status {
            StrictTokenPlanStatus::New => {
                match next {
                    StrictTokenPlanStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictTokenPlanStatus::Queued => {
                return next == StrictTokenPlanStatus::Done || next == StrictTokenPlanStatus::Failed;
            }
            StrictTokenPlanStatus::Running => {
                return next == StrictTokenPlanStatus::Done || next == StrictTokenPlanStatus::Failed;
            }
            StrictTokenPlanStatus::Done => {
                match next {
                    StrictTokenPlanStatus::Failed => {
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

    pub fn move_to(&mut self, next: StrictTokenPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict token plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "strict token plan", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.rate * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictTokenPlan {
        StrictTokenPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            port: self.port,
            timezone: self.timezone.clone(),
            rate: self.rate,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict token plan in status new.
pub fn new_strict_token_plan(code: &str, label: &str) -> StrictTokenPlan {
    StrictTokenPlan {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        port: 0,
        timezone: "".to_string(),
        rate: 0.0,
        host: "".to_string(),
        status: StrictTokenPlanStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_token_plan("code-591", "label-599");
        assert_eq!(item.status, StrictTokenPlanStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_strict_token_plan("code-591", "label-599");
        if item.move_to(StrictTokenPlanStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictTokenPlanStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_token_plan("code-591", "label-599");
        item.status = StrictTokenPlanStatus::Failed;
        assert!(
            item.move_to(StrictTokenPlanStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
