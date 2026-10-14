use super::*;

use acme_kit::text;

/// Where a probe record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeRecordStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ProbeRecordStatus {
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

impl std::fmt::Display for ProbeRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a probe record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRecord {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub maximum: f64,
    pub ttl_seconds: i64,
    pub status: ProbeRecordStatus,
}

impl ProbeRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeRecordStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeRecordStatus) -> bool {
        match self.status {
            ProbeRecordStatus::Idle => {
                match next {
                    ProbeRecordStatus::Ready | ProbeRecordStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeRecordStatus::Warming => {
                match next {
                    ProbeRecordStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeRecordStatus::Ready => {
                return next == ProbeRecordStatus::Draining || next == ProbeRecordStatus::Stopped;
            }
            ProbeRecordStatus::Draining => {
                return next == ProbeRecordStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeRecord {
        ProbeRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            maximum: self.maximum,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a probe record in status idle.
pub fn new_probe_record(code: &str, title: &str) -> ProbeRecord {
    ProbeRecord {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        maximum: 0.0,
        ttl_seconds: 0,
        status: ProbeRecordStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_probe_record("code-237", "title-153");
        assert_eq!(item.status, ProbeRecordStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_probe_record("code-237", "title-153");
        if item.move_to(ProbeRecordStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeRecordStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_probe_record("code-237", "title-153");
        item.status = ProbeRecordStatus::Stopped;
        assert!(item.move_to(ProbeRecordStatus::Idle).is_err(), "expected move to fail");
    }
}
