use super::*;

use acme_kit::text;

/// Where a primary rollout override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRolloutOverrideStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryRolloutOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryRolloutOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary rollout override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutOverride {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub discount: f64,
    pub duration_minutes: i64,
    pub port: i64,
    pub status: PrimaryRolloutOverrideStatus,
}

impl PrimaryRolloutOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRolloutOverrideStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRolloutOverrideStatus) -> bool {
        match self.status {
            PrimaryRolloutOverrideStatus::Created => {
                return next == PrimaryRolloutOverrideStatus::Validated
                    || next == PrimaryRolloutOverrideStatus::Published;
            }
            PrimaryRolloutOverrideStatus::Validated => {
                return next == PrimaryRolloutOverrideStatus::Published
                    || next == PrimaryRolloutOverrideStatus::Retired;
            }
            PrimaryRolloutOverrideStatus::Published => {
                match next {
                    PrimaryRolloutOverrideStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimaryRolloutOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rollout override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary rollout override", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.port as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRolloutOverride {
        PrimaryRolloutOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            discount: self.discount,
            duration_minutes: self.duration_minutes,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary rollout override in status created.
pub fn new_primary_rollout_override(code: &str, label: &str) -> PrimaryRolloutOverride {
    PrimaryRolloutOverride {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        discount: 0.0,
        duration_minutes: 0,
        port: 0,
        status: PrimaryRolloutOverrideStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_primary_rollout_override("code-620", "label-198");
        assert_eq!(item.status, PrimaryRolloutOverrideStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_primary_rollout_override("code-620", "label-198");
        if item.move_to(PrimaryRolloutOverrideStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRolloutOverrideStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_primary_rollout_override("code-620", "label-198");
        item.status = PrimaryRolloutOverrideStatus::Retired;
        assert!(
            item.move_to(PrimaryRolloutOverrideStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
