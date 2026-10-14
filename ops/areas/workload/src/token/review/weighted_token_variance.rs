use super::*;

use acme_kit::text;

/// Where a weighted token variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTokenVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedTokenVarianceStatus {
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

impl std::fmt::Display for WeightedTokenVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted token variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenVariance {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub category: String,
    pub fingerprint: String,
    pub maximum: f64,
    pub status: WeightedTokenVarianceStatus,
}

impl WeightedTokenVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTokenVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTokenVarianceStatus) -> bool {
        match self.status {
            WeightedTokenVarianceStatus::Idle => {
                match next {
                    WeightedTokenVarianceStatus::Warming | WeightedTokenVarianceStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTokenVarianceStatus::Warming => {
                match next {
                    WeightedTokenVarianceStatus::Draining | WeightedTokenVarianceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTokenVarianceStatus::Ready => {
                match next {
                    WeightedTokenVarianceStatus::Draining | WeightedTokenVarianceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTokenVarianceStatus::Draining => {
                return next == WeightedTokenVarianceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedTokenVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted token variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.maximum * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTokenVariance {
        WeightedTokenVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            category: self.category.clone(),
            fingerprint: self.fingerprint.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a weighted token variance in status idle.
pub fn new_weighted_token_variance(code: &str, name: &str) -> WeightedTokenVariance {
    WeightedTokenVariance {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        category: "".to_string(),
        fingerprint: "".to_string(),
        maximum: 0.0,
        status: WeightedTokenVarianceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_token_variance("code-129", "name-871");
        assert_eq!(item.status, WeightedTokenVarianceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_weighted_token_variance("code-129", "name-871");
        if item.move_to(WeightedTokenVarianceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTokenVarianceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_token_variance("code-129", "name-871");
        item.status = WeightedTokenVarianceStatus::Stopped;
        assert!(
            item.move_to(WeightedTokenVarianceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
