use super::*;

use acme_kit::text;

/// Where a tiered isolation estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredIsolationEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredIsolationEstimateStatus {
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

impl std::fmt::Display for TieredIsolationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered isolation estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationEstimate {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub category: String,
    pub minimum: f64,
    pub contact_email: String,
    pub country: String,
    pub quantity: i64,
    pub status: TieredIsolationEstimateStatus,
}

impl TieredIsolationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredIsolationEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredIsolationEstimateStatus) -> bool {
        match self.status {
            TieredIsolationEstimateStatus::Idle => {
                return next == TieredIsolationEstimateStatus::Draining;
            }
            TieredIsolationEstimateStatus::Warming => {
                match next {
                    TieredIsolationEstimateStatus::Ready | TieredIsolationEstimateStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredIsolationEstimateStatus::Ready => {
                match next {
                    TieredIsolationEstimateStatus::Draining | TieredIsolationEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredIsolationEstimateStatus::Draining => {
                match next {
                    TieredIsolationEstimateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TieredIsolationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered isolation estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered isolation estimate", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.minimum + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredIsolationEstimate {
        TieredIsolationEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            category: self.category.clone(),
            minimum: self.minimum,
            contact_email: self.contact_email.clone(),
            country: self.country.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tiered isolation estimate in status idle.
pub fn new_tiered_isolation_estimate(code: &str, title: &str) -> TieredIsolationEstimate {
    TieredIsolationEstimate {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        category: "".to_string(),
        minimum: 0.0,
        contact_email: "".to_string(),
        country: "".to_string(),
        quantity: 0,
        status: TieredIsolationEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_isolation_estimate("code-497", "title-281");
        assert_eq!(item.status, TieredIsolationEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_tiered_isolation_estimate("code-497", "title-281");
        if item.move_to(TieredIsolationEstimateStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredIsolationEstimateStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_isolation_estimate("code-497", "title-281");
        item.status = TieredIsolationEstimateStatus::Stopped;
        assert!(
            item.move_to(TieredIsolationEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
