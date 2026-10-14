use super::*;

use acme_kit::text;

/// Where a sink digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SinkDigestStatus {
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

impl std::fmt::Display for SinkDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One sink digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkDigest {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub capacity: i64,
    pub principal: String,
    pub summary: String,
    pub external_ref: String,
    pub status: SinkDigestStatus,
}

impl SinkDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkDigestStatus) -> bool {
        match self.status {
            SinkDigestStatus::Idle => {
                match next {
                    SinkDigestStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkDigestStatus::Warming => {
                match next {
                    SinkDigestStatus::Ready | SinkDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkDigestStatus::Ready => {
                match next {
                    SinkDigestStatus::Draining | SinkDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkDigestStatus::Draining => {
                match next {
                    SinkDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SinkDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "sink digest", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.capacity as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkDigest {
        SinkDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            capacity: self.capacity,
            principal: self.principal.clone(),
            summary: self.summary.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a sink digest in status idle.
pub fn new_sink_digest(code: &str, title: &str) -> SinkDigest {
    SinkDigest {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        capacity: 0,
        principal: "".to_string(),
        summary: "".to_string(),
        external_ref: "".to_string(),
        status: SinkDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_sink_digest("code-930", "title-746");
        assert_eq!(item.status, SinkDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_sink_digest("code-930", "title-746");
        if item.move_to(SinkDigestStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkDigestStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_sink_digest("code-930", "title-746");
        item.status = SinkDigestStatus::Stopped;
        assert!(item.move_to(SinkDigestStatus::Idle).is_err(), "expected move to fail");
    }
}
