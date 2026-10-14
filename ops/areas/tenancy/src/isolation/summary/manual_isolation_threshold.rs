use super::*;

use acme_kit::text;

/// Where a manual isolation threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualIsolationThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualIsolationThresholdStatus {
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

impl std::fmt::Display for ManualIsolationThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual isolation threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationThreshold {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub threshold: f64,
    pub audience: String,
    pub expires_at: i64,
    pub currency: String,
    pub checksum: String,
    pub status: ManualIsolationThresholdStatus,
}

impl ManualIsolationThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualIsolationThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualIsolationThresholdStatus) -> bool {
        match self.status {
            ManualIsolationThresholdStatus::Idle => {
                match next {
                    ManualIsolationThresholdStatus::Warming | ManualIsolationThresholdStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualIsolationThresholdStatus::Warming => {
                match next {
                    ManualIsolationThresholdStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualIsolationThresholdStatus::Ready => {
                match next {
                    ManualIsolationThresholdStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualIsolationThresholdStatus::Draining => {
                return next == ManualIsolationThresholdStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualIsolationThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual isolation threshold {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.expires_at as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualIsolationThreshold {
        ManualIsolationThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            threshold: self.threshold,
            audience: self.audience.clone(),
            expires_at: self.expires_at,
            currency: self.currency.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual isolation threshold in status idle.
pub fn new_manual_isolation_threshold(code: &str, name: &str) -> ManualIsolationThreshold {
    ManualIsolationThreshold {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        threshold: 0.0,
        audience: "".to_string(),
        expires_at: 0,
        currency: "".to_string(),
        checksum: "".to_string(),
        status: ManualIsolationThresholdStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_isolation_threshold("code-113", "name-35");
        assert_eq!(item.status, ManualIsolationThresholdStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_manual_isolation_threshold("code-113", "name-35");
        if item.move_to(ManualIsolationThresholdStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualIsolationThresholdStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_isolation_threshold("code-113", "name-35");
        item.status = ManualIsolationThresholdStatus::Stopped;
        assert!(
            item.move_to(ManualIsolationThresholdStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
