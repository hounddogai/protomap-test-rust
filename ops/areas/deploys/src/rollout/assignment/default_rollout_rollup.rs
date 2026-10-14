use super::*;

use acme_kit::text;

/// Where a default rollout rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRolloutRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultRolloutRollupStatus {
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

impl std::fmt::Display for DefaultRolloutRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default rollout rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutRollup {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub archived: bool,
    pub scope: String,
    pub quantity: i64,
    pub amount: f64,
    pub status: DefaultRolloutRollupStatus,
}

impl DefaultRolloutRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRolloutRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRolloutRollupStatus) -> bool {
        match self.status {
            DefaultRolloutRollupStatus::New => {
                return next == DefaultRolloutRollupStatus::Done;
            }
            DefaultRolloutRollupStatus::Queued => {
                match next {
                    DefaultRolloutRollupStatus::Running | DefaultRolloutRollupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRolloutRollupStatus::Running => {
                match next {
                    DefaultRolloutRollupStatus::Done | DefaultRolloutRollupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRolloutRollupStatus::Done => {
                return next == DefaultRolloutRollupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultRolloutRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default rollout rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quantity as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRolloutRollup {
        DefaultRolloutRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            archived: self.archived,
            scope: self.scope.clone(),
            quantity: self.quantity,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default rollout rollup in status new.
pub fn new_default_rollout_rollup(code: &str, name: &str) -> DefaultRolloutRollup {
    DefaultRolloutRollup {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        archived: false,
        scope: "".to_string(),
        quantity: 0,
        amount: 0.0,
        status: DefaultRolloutRollupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_rollout_rollup("code-343", "name-492");
        assert_eq!(item.status, DefaultRolloutRollupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_default_rollout_rollup("code-343", "name-492");
        if item.move_to(DefaultRolloutRollupStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRolloutRollupStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_rollout_rollup("code-343", "name-492");
        item.status = DefaultRolloutRollupStatus::Failed;
        assert!(
            item.move_to(DefaultRolloutRollupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
