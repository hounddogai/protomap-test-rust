use super::*;

use acme_kit::text;

/// Where a identity plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl IdentityPlanStatus {
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

impl std::fmt::Display for IdentityPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a identity plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityPlan {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub starts_at: i64,
    pub active: bool,
    pub status: IdentityPlanStatus,
}

impl IdentityPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityPlanStatus) -> bool {
        match self.status {
            IdentityPlanStatus::New => {
                return next == IdentityPlanStatus::Queued;
            }
            IdentityPlanStatus::Queued => {
                return next == IdentityPlanStatus::Running;
            }
            IdentityPlanStatus::Running => {
                match next {
                    IdentityPlanStatus::Done | IdentityPlanStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityPlanStatus::Done => {
                return next == IdentityPlanStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> IdentityPlan {
        IdentityPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            starts_at: self.starts_at,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a identity plan in status new.
pub fn new_identity_plan(code: &str, title: &str) -> IdentityPlan {
    IdentityPlan {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        starts_at: 0,
        active: false,
        status: IdentityPlanStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_identity_plan("code-808", "title-492");
        assert_eq!(item.status, IdentityPlanStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_identity_plan("code-808", "title-492");
        if item.move_to(IdentityPlanStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityPlanStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_identity_plan("code-808", "title-492");
        item.status = IdentityPlanStatus::Failed;
        assert!(item.move_to(IdentityPlanStatus::New).is_err(), "expected move to fail");
    }
}
