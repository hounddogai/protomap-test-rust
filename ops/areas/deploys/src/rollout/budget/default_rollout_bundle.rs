use super::*;

use acme_kit::text;

/// Where a default rollout bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRolloutBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultRolloutBundleStatus {
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

impl std::fmt::Display for DefaultRolloutBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default rollout bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBundle {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub sequence: i64,
    pub rate: f64,
    pub floor_value: f64,
    pub external_ref: String,
    pub status: DefaultRolloutBundleStatus,
}

impl DefaultRolloutBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRolloutBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRolloutBundleStatus) -> bool {
        match self.status {
            DefaultRolloutBundleStatus::Idle => {
                match next {
                    DefaultRolloutBundleStatus::Warming | DefaultRolloutBundleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRolloutBundleStatus::Warming => {
                match next {
                    DefaultRolloutBundleStatus::Ready | DefaultRolloutBundleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRolloutBundleStatus::Ready => {
                return next == DefaultRolloutBundleStatus::Draining || next == DefaultRolloutBundleStatus::Stopped;
            }
            DefaultRolloutBundleStatus::Draining => {
                match next {
                    DefaultRolloutBundleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultRolloutBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default rollout bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.floor_value * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRolloutBundle {
        DefaultRolloutBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            sequence: self.sequence,
            rate: self.rate,
            floor_value: self.floor_value,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default rollout bundle in status idle.
pub fn new_default_rollout_bundle(code: &str, title: &str) -> DefaultRolloutBundle {
    DefaultRolloutBundle {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        sequence: 0,
        rate: 0.0,
        floor_value: 0.0,
        external_ref: "".to_string(),
        status: DefaultRolloutBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_rollout_bundle("code-141", "title-12");
        assert_eq!(item.status, DefaultRolloutBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_default_rollout_bundle("code-141", "title-12");
        if item.move_to(DefaultRolloutBundleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRolloutBundleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_rollout_bundle("code-141", "title-12");
        item.status = DefaultRolloutBundleStatus::Stopped;
        assert!(
            item.move_to(DefaultRolloutBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
