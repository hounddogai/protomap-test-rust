use super::*;

use acme_kit::text;

/// Where a capped release segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedReleaseSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CappedReleaseSegmentStatus {
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

impl std::fmt::Display for CappedReleaseSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped release segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedReleaseSegment {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub contact_email: String,
    pub fingerprint: String,
    pub description: String,
    pub discount: f64,
    pub status: CappedReleaseSegmentStatus,
}

impl CappedReleaseSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedReleaseSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedReleaseSegmentStatus) -> bool {
        match self.status {
            CappedReleaseSegmentStatus::Idle => {
                return next == CappedReleaseSegmentStatus::Warming || next == CappedReleaseSegmentStatus::Ready;
            }
            CappedReleaseSegmentStatus::Warming => {
                match next {
                    CappedReleaseSegmentStatus::Ready | CappedReleaseSegmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedReleaseSegmentStatus::Ready => {
                return next == CappedReleaseSegmentStatus::Draining || next == CappedReleaseSegmentStatus::Stopped;
            }
            CappedReleaseSegmentStatus::Draining => {
                match next {
                    CappedReleaseSegmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: CappedReleaseSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped release segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.attempts as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedReleaseSegment {
        CappedReleaseSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            contact_email: self.contact_email.clone(),
            fingerprint: self.fingerprint.clone(),
            description: self.description.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped release segment in status idle.
pub fn new_capped_release_segment(code: &str, name: &str) -> CappedReleaseSegment {
    CappedReleaseSegment {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        contact_email: "".to_string(),
        fingerprint: "".to_string(),
        description: "".to_string(),
        discount: 0.0,
        status: CappedReleaseSegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_capped_release_segment("code-823", "name-151");
        assert_eq!(item.status, CappedReleaseSegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_capped_release_segment("code-823", "name-151");
        if item.move_to(CappedReleaseSegmentStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedReleaseSegmentStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_capped_release_segment("code-823", "name-151");
        item.status = CappedReleaseSegmentStatus::Stopped;
        assert!(
            item.move_to(CappedReleaseSegmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
