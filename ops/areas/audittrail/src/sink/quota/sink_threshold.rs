use super::*;

use acme_kit::text;

/// Where a sink threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkThresholdStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SinkThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for SinkThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkThreshold {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub active: bool,
    pub cipher: String,
    pub checksum: String,
    pub status: SinkThresholdStatus,
}

impl SinkThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkThresholdStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkThresholdStatus) -> bool {
        match self.status {
            SinkThresholdStatus::New => {
                match next {
                    SinkThresholdStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkThresholdStatus::Queued => {
                match next {
                    SinkThresholdStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkThresholdStatus::Running => {
                match next {
                    SinkThresholdStatus::Done | SinkThresholdStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkThresholdStatus::Done => {
                match next {
                    SinkThresholdStatus::Failed => {
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

    pub fn move_to(&mut self, next: SinkThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> SinkThreshold {
        SinkThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            active: self.active,
            cipher: self.cipher.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a sink threshold in status new.
pub fn new_sink_threshold(code: &str, label: &str) -> SinkThreshold {
    SinkThreshold {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        active: false,
        cipher: "".to_string(),
        checksum: "".to_string(),
        status: SinkThresholdStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_sink_threshold("code-986", "label-714");
        assert_eq!(item.status, SinkThresholdStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_sink_threshold("code-986", "label-714");
        if item.move_to(SinkThresholdStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkThresholdStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_sink_threshold("code-986", "label-714");
        item.status = SinkThresholdStatus::Failed;
        assert!(item.move_to(SinkThresholdStatus::New).is_err(), "expected move to fail");
    }
}
