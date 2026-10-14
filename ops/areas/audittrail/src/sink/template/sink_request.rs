use super::*;

use acme_kit::text;

/// Where a sink request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkRequestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SinkRequestStatus {
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

impl std::fmt::Display for SinkRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One sink request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkRequest {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub currency: String,
    pub version: i64,
    pub limit: i64,
    pub weight: f64,
    pub segment: String,
    pub status: SinkRequestStatus,
}

impl SinkRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkRequestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkRequestStatus) -> bool {
        match self.status {
            SinkRequestStatus::Idle => {
                return next == SinkRequestStatus::Draining;
            }
            SinkRequestStatus::Warming => {
                match next {
                    SinkRequestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkRequestStatus::Ready => {
                match next {
                    SinkRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkRequestStatus::Draining => {
                match next {
                    SinkRequestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SinkRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "sink request", self.code)
    }

    pub fn net_value(&self) -> f64 {
        (self.weight * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkRequest {
        SinkRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            currency: self.currency.clone(),
            version: self.version,
            limit: self.limit,
            weight: self.weight,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a sink request in status idle.
pub fn new_sink_request(code: &str, label: &str) -> SinkRequest {
    SinkRequest {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        currency: "".to_string(),
        version: 0,
        limit: 0,
        weight: 0.0,
        segment: "".to_string(),
        status: SinkRequestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_sink_request("code-753", "label-720");
        assert_eq!(item.status, SinkRequestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_sink_request("code-753", "label-720");
        if item.move_to(SinkRequestStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkRequestStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_sink_request("code-753", "label-720");
        item.status = SinkRequestStatus::Stopped;
        assert!(item.move_to(SinkRequestStatus::Idle).is_err(), "expected move to fail");
    }
}
