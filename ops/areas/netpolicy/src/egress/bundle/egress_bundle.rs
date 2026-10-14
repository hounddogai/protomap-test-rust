use super::*;

use acme_kit::text;

/// Where a egress bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressBundleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EgressBundleStatus {
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

impl std::fmt::Display for EgressBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a egress bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressBundle {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub minimum: f64,
    pub segment: String,
    pub pinned: bool,
    pub active: bool,
    pub duration_minutes: i64,
    pub status: EgressBundleStatus,
}

impl EgressBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressBundleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressBundleStatus) -> bool {
        match self.status {
            EgressBundleStatus::Planned => {
                return next == EgressBundleStatus::InProgress || next == EgressBundleStatus::Cancelled;
            }
            EgressBundleStatus::Scheduled => {
                return next == EgressBundleStatus::Completed;
            }
            EgressBundleStatus::InProgress => {
                match next {
                    EgressBundleStatus::Completed | EgressBundleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressBundleStatus::Completed => {
                match next {
                    EgressBundleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EgressBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "egress bundle", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.minimum + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressBundle {
        EgressBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            minimum: self.minimum,
            segment: self.segment.clone(),
            pinned: self.pinned,
            active: self.active,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a egress bundle in status planned.
pub fn new_egress_bundle(code: &str, name: &str) -> EgressBundle {
    EgressBundle {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        minimum: 0.0,
        segment: "".to_string(),
        pinned: false,
        active: false,
        duration_minutes: 0,
        status: EgressBundleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_egress_bundle("code-166", "name-848");
        assert_eq!(item.status, EgressBundleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_egress_bundle("code-166", "name-848");
        if item.move_to(EgressBundleStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressBundleStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_egress_bundle("code-166", "name-848");
        item.status = EgressBundleStatus::Cancelled;
        assert!(
            item.move_to(EgressBundleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
