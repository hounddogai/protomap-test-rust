use super::*;

use acme_kit::text;

/// Where a overage bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl OverageBundleStatus {
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

impl std::fmt::Display for OverageBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a overage bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageBundle {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub locale: String,
    pub namespace: String,
    pub status: OverageBundleStatus,
}

impl OverageBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageBundleStatus) -> bool {
        match self.status {
            OverageBundleStatus::Idle => {
                return next == OverageBundleStatus::Ready;
            }
            OverageBundleStatus::Warming => {
                return next == OverageBundleStatus::Stopped;
            }
            OverageBundleStatus::Ready => {
                return next == OverageBundleStatus::Draining;
            }
            OverageBundleStatus::Draining => {
                return next == OverageBundleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> OverageBundle {
        OverageBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            locale: self.locale.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a overage bundle in status idle.
pub fn new_overage_bundle(code: &str, label: &str) -> OverageBundle {
    OverageBundle {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        locale: "".to_string(),
        namespace: "".to_string(),
        status: OverageBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_overage_bundle("code-382", "label-685");
        assert_eq!(item.status, OverageBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_overage_bundle("code-382", "label-685");
        if item.move_to(OverageBundleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageBundleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_overage_bundle("code-382", "label-685");
        item.status = OverageBundleStatus::Stopped;
        assert!(
            item.move_to(OverageBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
