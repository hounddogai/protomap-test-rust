use super::*;

use acme_kit::text;

/// Where a seasonal canary threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalCanaryThresholdStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalCanaryThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalCanaryThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal canary threshold as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryThreshold {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub quantity: i64,
    pub ceiling: f64,
    pub channel: String,
    pub burst: i64,
    pub status: SeasonalCanaryThresholdStatus,
}

impl SeasonalCanaryThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalCanaryThresholdStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalCanaryThresholdStatus) -> bool {
        match self.status {
            SeasonalCanaryThresholdStatus::Requested => {
                match next {
                    SeasonalCanaryThresholdStatus::Granted | SeasonalCanaryThresholdStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalCanaryThresholdStatus::Granted => {
                return next == SeasonalCanaryThresholdStatus::Revoked;
            }
            SeasonalCanaryThresholdStatus::Revoked => {
                match next {
                    SeasonalCanaryThresholdStatus::Expired => {
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

    pub fn move_to(&mut self, next: SeasonalCanaryThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal canary threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.quantity as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalCanaryThreshold {
        SeasonalCanaryThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            quantity: self.quantity,
            ceiling: self.ceiling,
            channel: self.channel.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a seasonal canary threshold in status requested.
pub fn new_seasonal_canary_threshold(code: &str, name: &str) -> SeasonalCanaryThreshold {
    SeasonalCanaryThreshold {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        quantity: 0,
        ceiling: 0.0,
        channel: "".to_string(),
        burst: 0,
        status: SeasonalCanaryThresholdStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_canary_threshold("code-326", "name-636");
        assert_eq!(item.status, SeasonalCanaryThresholdStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_seasonal_canary_threshold("code-326", "name-636");
        if item.move_to(SeasonalCanaryThresholdStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalCanaryThresholdStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_canary_threshold("code-326", "name-636");
        item.status = SeasonalCanaryThresholdStatus::Expired;
        assert!(
            item.move_to(SeasonalCanaryThresholdStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
