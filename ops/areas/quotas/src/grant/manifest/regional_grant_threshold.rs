use super::*;

use acme_kit::text;

/// Where a regional grant threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalGrantThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalGrantThresholdStatus {
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

impl std::fmt::Display for RegionalGrantThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional grant threshold as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantThreshold {
    pub code: String,
    pub name: String,
    pub country: String,
    pub timezone: String,
    pub burst: i64,
    pub capacity: i64,
    pub threshold: f64,
    pub status: RegionalGrantThresholdStatus,
}

impl RegionalGrantThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalGrantThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalGrantThresholdStatus) -> bool {
        match self.status {
            RegionalGrantThresholdStatus::Idle => {
                return next == RegionalGrantThresholdStatus::Ready || next == RegionalGrantThresholdStatus::Stopped;
            }
            RegionalGrantThresholdStatus::Warming => {
                match next {
                    RegionalGrantThresholdStatus::Ready | RegionalGrantThresholdStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalGrantThresholdStatus::Ready => {
                match next {
                    RegionalGrantThresholdStatus::Draining | RegionalGrantThresholdStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalGrantThresholdStatus::Draining => {
                match next {
                    RegionalGrantThresholdStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionalGrantThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional grant threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional grant threshold", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.threshold + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalGrantThreshold {
        RegionalGrantThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            timezone: self.timezone.clone(),
            burst: self.burst,
            capacity: self.capacity,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a regional grant threshold in status idle.
pub fn new_regional_grant_threshold(code: &str, name: &str) -> RegionalGrantThreshold {
    RegionalGrantThreshold {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        timezone: "".to_string(),
        burst: 0,
        capacity: 0,
        threshold: 0.0,
        status: RegionalGrantThresholdStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_grant_threshold("code-172", "name-452");
        assert_eq!(item.status, RegionalGrantThresholdStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_regional_grant_threshold("code-172", "name-452");
        if item.move_to(RegionalGrantThresholdStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalGrantThresholdStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_grant_threshold("code-172", "name-452");
        item.status = RegionalGrantThresholdStatus::Stopped;
        assert!(
            item.move_to(RegionalGrantThresholdStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
