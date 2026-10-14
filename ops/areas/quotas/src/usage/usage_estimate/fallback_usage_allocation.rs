use super::*;

use acme_kit::text;

/// Where a fallback usage allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackUsageAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackUsageAllocationStatus {
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

impl std::fmt::Display for FallbackUsageAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback usage allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageAllocation {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub namespace: String,
    pub weight: f64,
    pub status: FallbackUsageAllocationStatus,
}

impl FallbackUsageAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackUsageAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackUsageAllocationStatus) -> bool {
        match self.status {
            FallbackUsageAllocationStatus::Planned => {
                return next == FallbackUsageAllocationStatus::Completed
                    || next == FallbackUsageAllocationStatus::Cancelled;
            }
            FallbackUsageAllocationStatus::Scheduled => {
                match next {
                    FallbackUsageAllocationStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackUsageAllocationStatus::InProgress => {
                return next == FallbackUsageAllocationStatus::Completed;
            }
            FallbackUsageAllocationStatus::Completed => {
                return next == FallbackUsageAllocationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackUsageAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback usage allocation {} cannot move from {} to {}",
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
        format!("{} {}", "fallback usage allocation", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.updated_at as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackUsageAllocation {
        FallbackUsageAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            namespace: self.namespace.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback usage allocation in status planned.
pub fn new_fallback_usage_allocation(code: &str, title: &str) -> FallbackUsageAllocation {
    FallbackUsageAllocation {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        namespace: "".to_string(),
        weight: 0.0,
        status: FallbackUsageAllocationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_usage_allocation("code-830", "title-514");
        assert_eq!(item.status, FallbackUsageAllocationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_fallback_usage_allocation("code-830", "title-514");
        if item.move_to(FallbackUsageAllocationStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackUsageAllocationStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_usage_allocation("code-830", "title-514");
        item.status = FallbackUsageAllocationStatus::Cancelled;
        assert!(
            item.move_to(FallbackUsageAllocationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
