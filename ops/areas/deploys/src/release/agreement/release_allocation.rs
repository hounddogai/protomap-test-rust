use super::*;

use acme_kit::text;

/// Where a release allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ReleaseAllocationStatus {
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

impl std::fmt::Display for ReleaseAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One release allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAllocation {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub updated_at: i64,
    pub visible: bool,
    pub rotation_days: i64,
    pub cluster: String,
    pub category: String,
    pub status: ReleaseAllocationStatus,
}

impl ReleaseAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseAllocationStatus) -> bool {
        match self.status {
            ReleaseAllocationStatus::New => {
                return next == ReleaseAllocationStatus::Done;
            }
            ReleaseAllocationStatus::Queued => {
                match next {
                    ReleaseAllocationStatus::Done | ReleaseAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseAllocationStatus::Running => {
                match next {
                    ReleaseAllocationStatus::Done | ReleaseAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseAllocationStatus::Done => {
                match next {
                    ReleaseAllocationStatus::Failed => {
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

    pub fn move_to(&mut self, next: ReleaseAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release allocation {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "release allocation", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.rotation_days as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseAllocation {
        ReleaseAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            updated_at: self.updated_at,
            visible: self.visible,
            rotation_days: self.rotation_days,
            cluster: self.cluster.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a release allocation in status new.
pub fn new_release_allocation(code: &str, title: &str) -> ReleaseAllocation {
    ReleaseAllocation {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        updated_at: 0,
        visible: false,
        rotation_days: 0,
        cluster: "".to_string(),
        category: "".to_string(),
        status: ReleaseAllocationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_release_allocation("code-773", "title-119");
        assert_eq!(item.status, ReleaseAllocationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_release_allocation("code-773", "title-119");
        if item.move_to(ReleaseAllocationStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseAllocationStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_release_allocation("code-773", "title-119");
        item.status = ReleaseAllocationStatus::Failed;
        assert!(
            item.move_to(ReleaseAllocationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
