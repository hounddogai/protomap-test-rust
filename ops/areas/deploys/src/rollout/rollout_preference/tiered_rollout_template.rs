use super::*;

use acme_kit::text;

/// Where a tiered rollout template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRolloutTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredRolloutTemplateStatus {
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

impl std::fmt::Display for TieredRolloutTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered rollout template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutTemplate {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub budget: f64,
    pub archived: bool,
    pub cost: f64,
    pub status: TieredRolloutTemplateStatus,
}

impl TieredRolloutTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRolloutTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRolloutTemplateStatus) -> bool {
        match self.status {
            TieredRolloutTemplateStatus::Idle => {
                return next == TieredRolloutTemplateStatus::Draining;
            }
            TieredRolloutTemplateStatus::Warming => {
                return next == TieredRolloutTemplateStatus::Ready || next == TieredRolloutTemplateStatus::Draining;
            }
            TieredRolloutTemplateStatus::Ready => {
                match next {
                    TieredRolloutTemplateStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRolloutTemplateStatus::Draining => {
                match next {
                    TieredRolloutTemplateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TieredRolloutTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered rollout template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.cost - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRolloutTemplate {
        TieredRolloutTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            budget: self.budget,
            archived: self.archived,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered rollout template in status idle.
pub fn new_tiered_rollout_template(code: &str, title: &str) -> TieredRolloutTemplate {
    TieredRolloutTemplate {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        budget: 0.0,
        archived: false,
        cost: 0.0,
        status: TieredRolloutTemplateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_rollout_template("code-10", "title-892");
        assert_eq!(item.status, TieredRolloutTemplateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_tiered_rollout_template("code-10", "title-892");
        if item.move_to(TieredRolloutTemplateStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRolloutTemplateStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_rollout_template("code-10", "title-892");
        item.status = TieredRolloutTemplateStatus::Stopped;
        assert!(
            item.move_to(TieredRolloutTemplateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
