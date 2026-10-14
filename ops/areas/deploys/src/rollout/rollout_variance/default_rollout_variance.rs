use super::*;

use acme_kit::text;

/// Where a default rollout variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRolloutVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultRolloutVarianceStatus {
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

impl std::fmt::Display for DefaultRolloutVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default rollout variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutVariance {
    pub code: String,
    pub title: String,
    pub category: String,
    pub created_at: i64,
    pub key_id: String,
    pub ratio: f64,
    pub quantity: i64,
    pub limit: i64,
    pub external_ref: String,
    pub name: String,
    pub rate: f64,
    pub status: DefaultRolloutVarianceStatus,
}

impl DefaultRolloutVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRolloutVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRolloutVarianceStatus) -> bool {
        match self.status {
            DefaultRolloutVarianceStatus::Idle => {
                return next == DefaultRolloutVarianceStatus::Warming;
            }
            DefaultRolloutVarianceStatus::Warming => {
                match next {
                    DefaultRolloutVarianceStatus::Ready | DefaultRolloutVarianceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRolloutVarianceStatus::Ready => {
                return next == DefaultRolloutVarianceStatus::Draining;
            }
            DefaultRolloutVarianceStatus::Draining => {
                return next == DefaultRolloutVarianceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultRolloutVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default rollout variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.created_at as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRolloutVariance {
        DefaultRolloutVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            created_at: self.created_at,
            key_id: self.key_id.clone(),
            ratio: self.ratio,
            quantity: self.quantity,
            limit: self.limit,
            external_ref: self.external_ref.clone(),
            name: self.name.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a default rollout variance in status idle.
pub fn new_default_rollout_variance(code: &str, title: &str) -> DefaultRolloutVariance {
    DefaultRolloutVariance {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        created_at: 0,
        key_id: "".to_string(),
        ratio: 0.0,
        quantity: 0,
        limit: 0,
        external_ref: "".to_string(),
        name: "".to_string(),
        rate: 0.0,
        status: DefaultRolloutVarianceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_rollout_variance("code-445", "title-501");
        assert_eq!(item.status, DefaultRolloutVarianceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_default_rollout_variance("code-445", "title-501");
        if item.move_to(DefaultRolloutVarianceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRolloutVarianceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_rollout_variance("code-445", "title-501");
        item.status = DefaultRolloutVarianceStatus::Stopped;
        assert!(
            item.move_to(DefaultRolloutVarianceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
