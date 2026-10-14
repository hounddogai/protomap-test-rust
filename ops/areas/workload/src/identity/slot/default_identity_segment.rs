use super::*;

use acme_kit::text;

/// Where a default identity segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultIdentitySegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultIdentitySegmentStatus {
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

impl std::fmt::Display for DefaultIdentitySegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default identity segment as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentitySegment {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub source: String,
    pub amount: f64,
    pub updated_at: i64,
    pub minimum: f64,
    pub status: DefaultIdentitySegmentStatus,
}

impl DefaultIdentitySegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultIdentitySegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultIdentitySegmentStatus) -> bool {
        match self.status {
            DefaultIdentitySegmentStatus::Idle => {
                return next == DefaultIdentitySegmentStatus::Ready;
            }
            DefaultIdentitySegmentStatus::Warming => {
                return next == DefaultIdentitySegmentStatus::Ready || next == DefaultIdentitySegmentStatus::Stopped;
            }
            DefaultIdentitySegmentStatus::Ready => {
                match next {
                    DefaultIdentitySegmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultIdentitySegmentStatus::Draining => {
                match next {
                    DefaultIdentitySegmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultIdentitySegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default identity segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default identity segment", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.amount - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultIdentitySegment {
        DefaultIdentitySegment {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            source: self.source.clone(),
            amount: self.amount,
            updated_at: self.updated_at,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a default identity segment in status idle.
pub fn new_default_identity_segment(code: &str, name: &str) -> DefaultIdentitySegment {
    DefaultIdentitySegment {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        source: "".to_string(),
        amount: 0.0,
        updated_at: 0,
        minimum: 0.0,
        status: DefaultIdentitySegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_identity_segment("code-844", "name-787");
        assert_eq!(item.status, DefaultIdentitySegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_default_identity_segment("code-844", "name-787");
        if item.move_to(DefaultIdentitySegmentStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultIdentitySegmentStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_identity_segment("code-844", "name-787");
        item.status = DefaultIdentitySegmentStatus::Stopped;
        assert!(
            item.move_to(DefaultIdentitySegmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
