use super::*;

use acme_kit::text;

/// Where a pending probe override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingProbeOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingProbeOverrideStatus {
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

impl std::fmt::Display for PendingProbeOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending probe override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeOverride {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub grace_days: i64,
    pub unit: String,
    pub region: String,
    pub status: PendingProbeOverrideStatus,
}

impl PendingProbeOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingProbeOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingProbeOverrideStatus) -> bool {
        match self.status {
            PendingProbeOverrideStatus::New => {
                match next {
                    PendingProbeOverrideStatus::Running | PendingProbeOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingProbeOverrideStatus::Queued => {
                return next == PendingProbeOverrideStatus::Running;
            }
            PendingProbeOverrideStatus::Running => {
                match next {
                    PendingProbeOverrideStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingProbeOverrideStatus::Done => {
                match next {
                    PendingProbeOverrideStatus::Failed => {
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

    pub fn move_to(&mut self, next: PendingProbeOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending probe override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> PendingProbeOverride {
        PendingProbeOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            grace_days: self.grace_days,
            unit: self.unit.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a pending probe override in status new.
pub fn new_pending_probe_override(code: &str, title: &str) -> PendingProbeOverride {
    PendingProbeOverride {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        grace_days: 0,
        unit: "".to_string(),
        region: "".to_string(),
        status: PendingProbeOverrideStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_probe_override("code-299", "title-446");
        assert_eq!(item.status, PendingProbeOverrideStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_pending_probe_override("code-299", "title-446");
        if item.move_to(PendingProbeOverrideStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingProbeOverrideStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_probe_override("code-299", "title-446");
        item.status = PendingProbeOverrideStatus::Failed;
        assert!(
            item.move_to(PendingProbeOverrideStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
