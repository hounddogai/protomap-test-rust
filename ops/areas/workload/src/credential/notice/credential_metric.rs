use super::*;

use acme_kit::text;

/// Where a credential metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CredentialMetricStatus {
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

impl std::fmt::Display for CredentialMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One credential metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialMetric {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub threshold: f64,
    pub host: String,
    pub status: CredentialMetricStatus,
}

impl CredentialMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialMetricStatus) -> bool {
        match self.status {
            CredentialMetricStatus::Idle => {
                return next == CredentialMetricStatus::Warming || next == CredentialMetricStatus::Draining;
            }
            CredentialMetricStatus::Warming => {
                match next {
                    CredentialMetricStatus::Ready | CredentialMetricStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialMetricStatus::Ready => {
                match next {
                    CredentialMetricStatus::Draining | CredentialMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialMetricStatus::Draining => {
                match next {
                    CredentialMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: CredentialMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "credential metric", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.threshold + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialMetric {
        CredentialMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            threshold: self.threshold,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a credential metric in status idle.
pub fn new_credential_metric(code: &str, name: &str) -> CredentialMetric {
    CredentialMetric {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        threshold: 0.0,
        host: "".to_string(),
        status: CredentialMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_credential_metric("code-3", "name-633");
        assert_eq!(item.status, CredentialMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_credential_metric("code-3", "name-633");
        if item.move_to(CredentialMetricStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialMetricStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_credential_metric("code-3", "name-633");
        item.status = CredentialMetricStatus::Stopped;
        assert!(
            item.move_to(CredentialMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
