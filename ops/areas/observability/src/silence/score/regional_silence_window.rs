use super::*;

use acme_kit::text;

/// Where a regional silence window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSilenceWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalSilenceWindowStatus {
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

impl std::fmt::Display for RegionalSilenceWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional silence window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceWindow {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub amount: f64,
    pub currency: String,
    pub status: RegionalSilenceWindowStatus,
}

impl RegionalSilenceWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSilenceWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSilenceWindowStatus) -> bool {
        match self.status {
            RegionalSilenceWindowStatus::Idle => {
                match next {
                    RegionalSilenceWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSilenceWindowStatus::Warming => {
                return next == RegionalSilenceWindowStatus::Draining;
            }
            RegionalSilenceWindowStatus::Ready => {
                return next == RegionalSilenceWindowStatus::Draining;
            }
            RegionalSilenceWindowStatus::Draining => {
                return next == RegionalSilenceWindowStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalSilenceWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional silence window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> RegionalSilenceWindow {
        RegionalSilenceWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            amount: self.amount,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional silence window in status idle.
pub fn new_regional_silence_window(code: &str, name: &str) -> RegionalSilenceWindow {
    RegionalSilenceWindow {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        amount: 0.0,
        currency: "".to_string(),
        status: RegionalSilenceWindowStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_silence_window("code-504", "name-675");
        assert_eq!(item.status, RegionalSilenceWindowStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_regional_silence_window("code-504", "name-675");
        if item.move_to(RegionalSilenceWindowStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalSilenceWindowStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_silence_window("code-504", "name-675");
        item.status = RegionalSilenceWindowStatus::Stopped;
        assert!(
            item.move_to(RegionalSilenceWindowStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
