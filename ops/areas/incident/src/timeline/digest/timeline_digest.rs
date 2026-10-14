use super::*;

use acme_kit::text;

/// Where a timeline digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TimelineDigestStatus {
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

impl std::fmt::Display for TimelineDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDigest {
    pub code: String,
    pub title: String,
    pub country: String,
    pub label: String,
    pub threshold: f64,
    pub quantity: i64,
    pub notes: String,
    pub status: TimelineDigestStatus,
}

impl TimelineDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineDigestStatus) -> bool {
        match self.status {
            TimelineDigestStatus::Idle => {
                match next {
                    TimelineDigestStatus::Warming | TimelineDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineDigestStatus::Warming => {
                return next == TimelineDigestStatus::Ready;
            }
            TimelineDigestStatus::Ready => {
                return next == TimelineDigestStatus::Draining || next == TimelineDigestStatus::Stopped;
            }
            TimelineDigestStatus::Draining => {
                match next {
                    TimelineDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TimelineDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.quantity as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineDigest {
        TimelineDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            quantity: self.quantity,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a timeline digest in status idle.
pub fn new_timeline_digest(code: &str, title: &str) -> TimelineDigest {
    TimelineDigest {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        label: "".to_string(),
        threshold: 0.0,
        quantity: 0,
        notes: "".to_string(),
        status: TimelineDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_timeline_digest("code-707", "title-524");
        assert_eq!(item.status, TimelineDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_timeline_digest("code-707", "title-524");
        if item.move_to(TimelineDigestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineDigestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_timeline_digest("code-707", "title-524");
        item.status = TimelineDigestStatus::Stopped;
        assert!(
            item.move_to(TimelineDigestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
