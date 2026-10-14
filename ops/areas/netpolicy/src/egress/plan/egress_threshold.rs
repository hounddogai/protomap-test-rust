use super::*;

use acme_kit::text;

/// Where a egress threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EgressThresholdStatus {
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

impl std::fmt::Display for EgressThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One egress threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressThreshold {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub maximum: f64,
    pub sequence: i64,
    pub cipher: String,
    pub floor_value: f64,
    pub window_days: i64,
    pub port: i64,
    pub status: EgressThresholdStatus,
}

impl EgressThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressThresholdStatus) -> bool {
        match self.status {
            EgressThresholdStatus::Idle => {
                match next {
                    EgressThresholdStatus::Warming | EgressThresholdStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressThresholdStatus::Warming => {
                match next {
                    EgressThresholdStatus::Ready | EgressThresholdStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressThresholdStatus::Ready => {
                return next == EgressThresholdStatus::Draining;
            }
            EgressThresholdStatus::Draining => {
                return next == EgressThresholdStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.floor_value * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressThreshold {
        EgressThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            maximum: self.maximum,
            sequence: self.sequence,
            cipher: self.cipher.clone(),
            floor_value: self.floor_value,
            window_days: self.window_days,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a egress threshold in status idle.
pub fn new_egress_threshold(code: &str, title: &str) -> EgressThreshold {
    EgressThreshold {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        maximum: 0.0,
        sequence: 0,
        cipher: "".to_string(),
        floor_value: 0.0,
        window_days: 0,
        port: 0,
        status: EgressThresholdStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_egress_threshold("code-325", "title-294");
        assert_eq!(item.status, EgressThresholdStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_egress_threshold("code-325", "title-294");
        if item.move_to(EgressThresholdStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressThresholdStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_egress_threshold("code-325", "title-294");
        item.status = EgressThresholdStatus::Stopped;
        assert!(
            item.move_to(EgressThresholdStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
