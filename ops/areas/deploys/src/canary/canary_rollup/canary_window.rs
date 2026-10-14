use super::*;

use acme_kit::text;

/// Where a canary window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CanaryWindowStatus {
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

impl std::fmt::Display for CanaryWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryWindow {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub reviewer: String,
    pub host: String,
    pub channel: String,
    pub principal: String,
    pub country: String,
    pub status: CanaryWindowStatus,
}

impl CanaryWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryWindowStatus) -> bool {
        match self.status {
            CanaryWindowStatus::Idle => {
                match next {
                    CanaryWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryWindowStatus::Warming => {
                return next == CanaryWindowStatus::Ready;
            }
            CanaryWindowStatus::Ready => {
                return next == CanaryWindowStatus::Draining || next == CanaryWindowStatus::Stopped;
            }
            CanaryWindowStatus::Draining => {
                return next == CanaryWindowStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> CanaryWindow {
        CanaryWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            reviewer: self.reviewer.clone(),
            host: self.host.clone(),
            channel: self.channel.clone(),
            principal: self.principal.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a canary window in status idle.
pub fn new_canary_window(code: &str, name: &str) -> CanaryWindow {
    CanaryWindow {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        reviewer: "".to_string(),
        host: "".to_string(),
        channel: "".to_string(),
        principal: "".to_string(),
        country: "".to_string(),
        status: CanaryWindowStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_canary_window("code-867", "name-635");
        assert_eq!(item.status, CanaryWindowStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_canary_window("code-867", "name-635");
        if item.move_to(CanaryWindowStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryWindowStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_canary_window("code-867", "name-635");
        item.status = CanaryWindowStatus::Stopped;
        assert!(item.move_to(CanaryWindowStatus::Idle).is_err(), "expected move to fail");
    }
}
