use super::*;

use acme_kit::text;

/// Where a primary action item baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryActionItemBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryActionItemBaselineStatus {
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

impl std::fmt::Display for PrimaryActionItemBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary action item baseline as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBaseline {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub priority: i64,
    pub grace_days: i64,
    pub country: String,
    pub updated_at: i64,
    pub ceiling: f64,
    pub pinned: bool,
    pub status: PrimaryActionItemBaselineStatus,
}

impl PrimaryActionItemBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryActionItemBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryActionItemBaselineStatus) -> bool {
        match self.status {
            PrimaryActionItemBaselineStatus::Idle => {
                return next == PrimaryActionItemBaselineStatus::Ready;
            }
            PrimaryActionItemBaselineStatus::Warming => {
                match next {
                    PrimaryActionItemBaselineStatus::Ready | PrimaryActionItemBaselineStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryActionItemBaselineStatus::Ready => {
                return next == PrimaryActionItemBaselineStatus::Draining
                    || next == PrimaryActionItemBaselineStatus::Stopped;
            }
            PrimaryActionItemBaselineStatus::Draining => {
                return next == PrimaryActionItemBaselineStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryActionItemBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary action item baseline {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ceiling + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryActionItemBaseline {
        PrimaryActionItemBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            priority: self.priority,
            grace_days: self.grace_days,
            country: self.country.clone(),
            updated_at: self.updated_at,
            ceiling: self.ceiling,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a primary action item baseline in status idle.
pub fn new_primary_action_item_baseline(code: &str, title: &str) -> PrimaryActionItemBaseline {
    PrimaryActionItemBaseline {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        priority: 0,
        grace_days: 0,
        country: "".to_string(),
        updated_at: 0,
        ceiling: 0.0,
        pinned: false,
        status: PrimaryActionItemBaselineStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_action_item_baseline("code-933", "title-185");
        assert_eq!(item.status, PrimaryActionItemBaselineStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_primary_action_item_baseline("code-933", "title-185");
        if item.move_to(PrimaryActionItemBaselineStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryActionItemBaselineStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_action_item_baseline("code-933", "title-185");
        item.status = PrimaryActionItemBaselineStatus::Stopped;
        assert!(
            item.move_to(PrimaryActionItemBaselineStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
