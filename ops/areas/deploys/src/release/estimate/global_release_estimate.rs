use super::*;

use acme_kit::text;

/// Where a global release estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalReleaseEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalReleaseEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for GlobalReleaseEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global release estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEstimate {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub cipher: String,
    pub title: String,
    pub tenant_id: String,
    pub archived: bool,
    pub status: GlobalReleaseEstimateStatus,
}

impl GlobalReleaseEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalReleaseEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalReleaseEstimateStatus) -> bool {
        match self.status {
            GlobalReleaseEstimateStatus::Planned => {
                return next == GlobalReleaseEstimateStatus::InProgress
                    || next == GlobalReleaseEstimateStatus::Completed;
            }
            GlobalReleaseEstimateStatus::Scheduled => {
                return next == GlobalReleaseEstimateStatus::InProgress;
            }
            GlobalReleaseEstimateStatus::InProgress => {
                match next {
                    GlobalReleaseEstimateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReleaseEstimateStatus::Completed => {
                match next {
                    GlobalReleaseEstimateStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: GlobalReleaseEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global release estimate {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> GlobalReleaseEstimate {
        GlobalReleaseEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            cipher: self.cipher.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global release estimate in status planned.
pub fn new_global_release_estimate(code: &str, name: &str) -> GlobalReleaseEstimate {
    GlobalReleaseEstimate {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        cipher: "".to_string(),
        title: "".to_string(),
        tenant_id: "".to_string(),
        archived: false,
        status: GlobalReleaseEstimateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_release_estimate("code-538", "name-769");
        assert_eq!(item.status, GlobalReleaseEstimateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_global_release_estimate("code-538", "name-769");
        if item.move_to(GlobalReleaseEstimateStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalReleaseEstimateStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_release_estimate("code-538", "name-769");
        item.status = GlobalReleaseEstimateStatus::Cancelled;
        assert!(
            item.move_to(GlobalReleaseEstimateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
