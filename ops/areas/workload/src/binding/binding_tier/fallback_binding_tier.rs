use super::*;

use acme_kit::text;

/// Where a fallback binding tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackBindingTierStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackBindingTierStatus {
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

impl std::fmt::Display for FallbackBindingTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback binding tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingTier {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub external_ref: String,
    pub updated_at: i64,
    pub archived: bool,
    pub rotation_days: i64,
    pub status: FallbackBindingTierStatus,
}

impl FallbackBindingTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackBindingTierStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackBindingTierStatus) -> bool {
        match self.status {
            FallbackBindingTierStatus::New => {
                return next == FallbackBindingTierStatus::Running || next == FallbackBindingTierStatus::Done;
            }
            FallbackBindingTierStatus::Queued => {
                match next {
                    FallbackBindingTierStatus::Running | FallbackBindingTierStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackBindingTierStatus::Running => {
                return next == FallbackBindingTierStatus::Failed;
            }
            FallbackBindingTierStatus::Done => {
                match next {
                    FallbackBindingTierStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackBindingTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback binding tier {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.window_days as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackBindingTier {
        FallbackBindingTier {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            external_ref: self.external_ref.clone(),
            updated_at: self.updated_at,
            archived: self.archived,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback binding tier in status new.
pub fn new_fallback_binding_tier(code: &str, label: &str) -> FallbackBindingTier {
    FallbackBindingTier {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        external_ref: "".to_string(),
        updated_at: 0,
        archived: false,
        rotation_days: 0,
        status: FallbackBindingTierStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_binding_tier("code-666", "label-125");
        assert_eq!(item.status, FallbackBindingTierStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_fallback_binding_tier("code-666", "label-125");
        if item.move_to(FallbackBindingTierStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackBindingTierStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_binding_tier("code-666", "label-125");
        item.status = FallbackBindingTierStatus::Failed;
        assert!(
            item.move_to(FallbackBindingTierStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
