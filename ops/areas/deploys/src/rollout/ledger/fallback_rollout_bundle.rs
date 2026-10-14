use super::*;

use acme_kit::text;

/// Where a fallback rollout bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRolloutBundleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackRolloutBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackRolloutBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback rollout bundle as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutBundle {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub discount: f64,
    pub batch_size: i64,
    pub status: FallbackRolloutBundleStatus,
}

impl FallbackRolloutBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRolloutBundleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRolloutBundleStatus) -> bool {
        match self.status {
            FallbackRolloutBundleStatus::Created => {
                return next == FallbackRolloutBundleStatus::Published || next == FallbackRolloutBundleStatus::Retired;
            }
            FallbackRolloutBundleStatus::Validated => {
                return next == FallbackRolloutBundleStatus::Published || next == FallbackRolloutBundleStatus::Retired;
            }
            FallbackRolloutBundleStatus::Published => {
                match next {
                    FallbackRolloutBundleStatus::Retired => {
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

    pub fn move_to(&mut self, next: FallbackRolloutBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback rollout bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback rollout bundle", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.discount + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRolloutBundle {
        FallbackRolloutBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            discount: self.discount,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback rollout bundle in status created.
pub fn new_fallback_rollout_bundle(code: &str, name: &str) -> FallbackRolloutBundle {
    FallbackRolloutBundle {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        discount: 0.0,
        batch_size: 0,
        status: FallbackRolloutBundleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_rollout_bundle("code-87", "name-150");
        assert_eq!(item.status, FallbackRolloutBundleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_fallback_rollout_bundle("code-87", "name-150");
        if item.move_to(FallbackRolloutBundleStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRolloutBundleStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_rollout_bundle("code-87", "name-150");
        item.status = FallbackRolloutBundleStatus::Retired;
        assert!(
            item.move_to(FallbackRolloutBundleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
