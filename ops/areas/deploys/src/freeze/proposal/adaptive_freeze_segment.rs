use super::*;

use acme_kit::text;

/// Where a adaptive freeze segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveFreezeSegmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveFreezeSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveFreezeSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive freeze segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeSegment {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub contact_email: String,
    pub burst: i64,
    pub visible: bool,
    pub priority: i64,
    pub status: AdaptiveFreezeSegmentStatus,
}

impl AdaptiveFreezeSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveFreezeSegmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveFreezeSegmentStatus) -> bool {
        match self.status {
            AdaptiveFreezeSegmentStatus::Requested => {
                match next {
                    AdaptiveFreezeSegmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFreezeSegmentStatus::Granted => {
                return next == AdaptiveFreezeSegmentStatus::Revoked || next == AdaptiveFreezeSegmentStatus::Expired;
            }
            AdaptiveFreezeSegmentStatus::Revoked => {
                match next {
                    AdaptiveFreezeSegmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: AdaptiveFreezeSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive freeze segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive freeze segment", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.priority as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveFreezeSegment {
        AdaptiveFreezeSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            contact_email: self.contact_email.clone(),
            burst: self.burst,
            visible: self.visible,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive freeze segment in status requested.
pub fn new_adaptive_freeze_segment(code: &str, name: &str) -> AdaptiveFreezeSegment {
    AdaptiveFreezeSegment {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        contact_email: "".to_string(),
        burst: 0,
        visible: false,
        priority: 0,
        status: AdaptiveFreezeSegmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_adaptive_freeze_segment("code-589", "name-761");
        assert_eq!(item.status, AdaptiveFreezeSegmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_adaptive_freeze_segment("code-589", "name-761");
        if item.move_to(AdaptiveFreezeSegmentStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveFreezeSegmentStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_adaptive_freeze_segment("code-589", "name-761");
        item.status = AdaptiveFreezeSegmentStatus::Expired;
        assert!(
            item.move_to(AdaptiveFreezeSegmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
