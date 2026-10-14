use super::*;

use acme_kit::text;

/// Where a approval bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ApprovalBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ApprovalBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One approval bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBundle {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub segment: String,
    pub quota_units: i64,
    pub revision: i64,
    pub maximum: f64,
    pub status: ApprovalBundleStatus,
}

impl ApprovalBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalBundleStatus) -> bool {
        match self.status {
            ApprovalBundleStatus::Idle => {
                match next {
                    ApprovalBundleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalBundleStatus::Warming => {
                return next == ApprovalBundleStatus::Ready;
            }
            ApprovalBundleStatus::Ready => {
                return next == ApprovalBundleStatus::Draining;
            }
            ApprovalBundleStatus::Draining => {
                return next == ApprovalBundleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "approval bundle", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.cost * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalBundle {
        ApprovalBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            segment: self.segment.clone(),
            quota_units: self.quota_units,
            revision: self.revision,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a approval bundle in status idle.
pub fn new_approval_bundle(code: &str, label: &str) -> ApprovalBundle {
    ApprovalBundle {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        segment: "".to_string(),
        quota_units: 0,
        revision: 0,
        maximum: 0.0,
        status: ApprovalBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_approval_bundle("code-144", "label-773");
        assert_eq!(item.status, ApprovalBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_approval_bundle("code-144", "label-773");
        if item.move_to(ApprovalBundleStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalBundleStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_approval_bundle("code-144", "label-773");
        item.status = ApprovalBundleStatus::Stopped;
        assert!(
            item.move_to(ApprovalBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
