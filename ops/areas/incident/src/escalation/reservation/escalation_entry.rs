use super::*;

use acme_kit::text;

/// Where a escalation entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EscalationEntryStatus {
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

impl std::fmt::Display for EscalationEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation entry as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationEntry {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub ratio: f64,
    pub priority: i64,
    pub burst: i64,
    pub country: String,
    pub rate: f64,
    pub status: EscalationEntryStatus,
}

impl EscalationEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationEntryStatus) -> bool {
        match self.status {
            EscalationEntryStatus::Idle => {
                match next {
                    EscalationEntryStatus::Ready | EscalationEntryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationEntryStatus::Warming => {
                match next {
                    EscalationEntryStatus::Ready | EscalationEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationEntryStatus::Ready => {
                match next {
                    EscalationEntryStatus::Draining | EscalationEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationEntryStatus::Draining => {
                return next == EscalationEntryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "escalation entry", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.rate - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationEntry {
        EscalationEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            ratio: self.ratio,
            priority: self.priority,
            burst: self.burst,
            country: self.country.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a escalation entry in status idle.
pub fn new_escalation_entry(code: &str, label: &str) -> EscalationEntry {
    EscalationEntry {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        ratio: 0.0,
        priority: 0,
        burst: 0,
        country: "".to_string(),
        rate: 0.0,
        status: EscalationEntryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_escalation_entry("code-765", "label-306");
        assert_eq!(item.status, EscalationEntryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_escalation_entry("code-765", "label-306");
        if item.move_to(EscalationEntryStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationEntryStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_escalation_entry("code-765", "label-306");
        item.status = EscalationEntryStatus::Stopped;
        assert!(
            item.move_to(EscalationEntryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
