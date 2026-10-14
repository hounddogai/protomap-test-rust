use super::*;

use acme_kit::text;

/// Where a global release allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalReleaseAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalReleaseAllocationStatus {
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

impl std::fmt::Display for GlobalReleaseAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global release allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAllocation {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub fingerprint: String,
    pub batch_size: i64,
    pub ceiling: f64,
    pub weight: f64,
    pub checksum: String,
    pub channel: String,
    pub status: GlobalReleaseAllocationStatus,
}

impl GlobalReleaseAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalReleaseAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalReleaseAllocationStatus) -> bool {
        match self.status {
            GlobalReleaseAllocationStatus::Planned => {
                match next {
                    GlobalReleaseAllocationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReleaseAllocationStatus::Scheduled => {
                match next {
                    GlobalReleaseAllocationStatus::Completed | GlobalReleaseAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReleaseAllocationStatus::InProgress => {
                return next == GlobalReleaseAllocationStatus::Completed
                    || next == GlobalReleaseAllocationStatus::Cancelled;
            }
            GlobalReleaseAllocationStatus::Completed => {
                return next == GlobalReleaseAllocationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalReleaseAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global release allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ceiling + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalReleaseAllocation {
        GlobalReleaseAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            fingerprint: self.fingerprint.clone(),
            batch_size: self.batch_size,
            ceiling: self.ceiling,
            weight: self.weight,
            checksum: self.checksum.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a global release allocation in status planned.
pub fn new_global_release_allocation(code: &str, label: &str) -> GlobalReleaseAllocation {
    GlobalReleaseAllocation {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        fingerprint: "".to_string(),
        batch_size: 0,
        ceiling: 0.0,
        weight: 0.0,
        checksum: "".to_string(),
        channel: "".to_string(),
        status: GlobalReleaseAllocationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_release_allocation("code-926", "label-742");
        assert_eq!(item.status, GlobalReleaseAllocationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_global_release_allocation("code-926", "label-742");
        if item.move_to(GlobalReleaseAllocationStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalReleaseAllocationStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_release_allocation("code-926", "label-742");
        item.status = GlobalReleaseAllocationStatus::Cancelled;
        assert!(
            item.move_to(GlobalReleaseAllocationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
