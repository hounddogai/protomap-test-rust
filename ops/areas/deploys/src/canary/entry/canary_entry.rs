use super::*;

use acme_kit::text;

/// Where a canary entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CanaryEntryStatus {
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

impl std::fmt::Display for CanaryEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryEntry {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub reviewer: String,
    pub source: String,
    pub window_days: i64,
    pub locale: String,
    pub phone: String,
    pub status: CanaryEntryStatus,
}

impl CanaryEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryEntryStatus) -> bool {
        match self.status {
            CanaryEntryStatus::Idle => {
                match next {
                    CanaryEntryStatus::Warming | CanaryEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryEntryStatus::Warming => {
                match next {
                    CanaryEntryStatus::Ready | CanaryEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryEntryStatus::Ready => {
                match next {
                    CanaryEntryStatus::Draining | CanaryEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryEntryStatus::Draining => {
                match next {
                    CanaryEntryStatus::Stopped => {
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

    pub fn move_to(&mut self, next: CanaryEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> CanaryEntry {
        CanaryEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            reviewer: self.reviewer.clone(),
            source: self.source.clone(),
            window_days: self.window_days,
            locale: self.locale.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a canary entry in status idle.
pub fn new_canary_entry(code: &str, label: &str) -> CanaryEntry {
    CanaryEntry {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        reviewer: "".to_string(),
        source: "".to_string(),
        window_days: 0,
        locale: "".to_string(),
        phone: "".to_string(),
        status: CanaryEntryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_canary_entry("code-797", "label-800");
        assert_eq!(item.status, CanaryEntryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_canary_entry("code-797", "label-800");
        if item.move_to(CanaryEntryStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryEntryStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_canary_entry("code-797", "label-800");
        item.status = CanaryEntryStatus::Stopped;
        assert!(item.move_to(CanaryEntryStatus::Idle).is_err(), "expected move to fail");
    }
}
