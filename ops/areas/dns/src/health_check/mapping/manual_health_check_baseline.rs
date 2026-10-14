use super::*;

use acme_kit::text;

/// Where a manual health check baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualHealthCheckBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualHealthCheckBaselineStatus {
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

impl std::fmt::Display for ManualHealthCheckBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual health check baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckBaseline {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub maximum: f64,
    pub score: f64,
    pub status: ManualHealthCheckBaselineStatus,
}

impl ManualHealthCheckBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualHealthCheckBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualHealthCheckBaselineStatus) -> bool {
        match self.status {
            ManualHealthCheckBaselineStatus::Idle => {
                match next {
                    ManualHealthCheckBaselineStatus::Ready | ManualHealthCheckBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualHealthCheckBaselineStatus::Warming => {
                return next == ManualHealthCheckBaselineStatus::Ready
                    || next == ManualHealthCheckBaselineStatus::Draining;
            }
            ManualHealthCheckBaselineStatus::Ready => {
                match next {
                    ManualHealthCheckBaselineStatus::Draining | ManualHealthCheckBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualHealthCheckBaselineStatus::Draining => {
                return next == ManualHealthCheckBaselineStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualHealthCheckBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual health check baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual health check baseline", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.minimum + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualHealthCheckBaseline {
        ManualHealthCheckBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            maximum: self.maximum,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual health check baseline in status idle.
pub fn new_manual_health_check_baseline(code: &str, name: &str) -> ManualHealthCheckBaseline {
    ManualHealthCheckBaseline {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        maximum: 0.0,
        score: 0.0,
        status: ManualHealthCheckBaselineStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_health_check_baseline("code-370", "name-432");
        assert_eq!(item.status, ManualHealthCheckBaselineStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_manual_health_check_baseline("code-370", "name-432");
        if item.move_to(ManualHealthCheckBaselineStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualHealthCheckBaselineStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_health_check_baseline("code-370", "name-432");
        item.status = ManualHealthCheckBaselineStatus::Stopped;
        assert!(
            item.move_to(ManualHealthCheckBaselineStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
