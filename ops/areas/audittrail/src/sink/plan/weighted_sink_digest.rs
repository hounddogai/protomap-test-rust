use super::*;

use acme_kit::text;

/// Where a weighted sink digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedSinkDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedSinkDigestStatus {
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

impl std::fmt::Display for WeightedSinkDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted sink digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSinkDigest {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub description: String,
    pub principal: String,
    pub attempts: i64,
    pub budget: f64,
    pub port: i64,
    pub batch_size: i64,
    pub summary: String,
    pub burst: i64,
    pub status: WeightedSinkDigestStatus,
}

impl WeightedSinkDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedSinkDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedSinkDigestStatus) -> bool {
        match self.status {
            WeightedSinkDigestStatus::Idle => {
                match next {
                    WeightedSinkDigestStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedSinkDigestStatus::Warming => {
                return next == WeightedSinkDigestStatus::Stopped;
            }
            WeightedSinkDigestStatus::Ready => {
                match next {
                    WeightedSinkDigestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedSinkDigestStatus::Draining => {
                match next {
                    WeightedSinkDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: WeightedSinkDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted sink digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.attempts as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedSinkDigest {
        WeightedSinkDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            description: self.description.clone(),
            principal: self.principal.clone(),
            attempts: self.attempts,
            budget: self.budget,
            port: self.port,
            batch_size: self.batch_size,
            summary: self.summary.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted sink digest in status idle.
pub fn new_weighted_sink_digest(code: &str, label: &str) -> WeightedSinkDigest {
    WeightedSinkDigest {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        description: "".to_string(),
        principal: "".to_string(),
        attempts: 0,
        budget: 0.0,
        port: 0,
        batch_size: 0,
        summary: "".to_string(),
        burst: 0,
        status: WeightedSinkDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_sink_digest("code-94", "label-427");
        assert_eq!(item.status, WeightedSinkDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_weighted_sink_digest("code-94", "label-427");
        if item.move_to(WeightedSinkDigestStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedSinkDigestStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_sink_digest("code-94", "label-427");
        item.status = WeightedSinkDigestStatus::Stopped;
        assert!(
            item.move_to(WeightedSinkDigestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
