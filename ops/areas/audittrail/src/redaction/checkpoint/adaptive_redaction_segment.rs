use super::*;

use acme_kit::text;

/// Where a adaptive redaction segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRedactionSegmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveRedactionSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveRedactionSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive redaction segment as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRedactionSegment {
    pub code: String,
    pub name: String,
    pub source: String,
    pub timezone: String,
    pub pinned: bool,
    pub floor_value: f64,
    pub status: AdaptiveRedactionSegmentStatus,
}

impl AdaptiveRedactionSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRedactionSegmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRedactionSegmentStatus) -> bool {
        match self.status {
            AdaptiveRedactionSegmentStatus::Requested => {
                return next == AdaptiveRedactionSegmentStatus::Revoked;
            }
            AdaptiveRedactionSegmentStatus::Granted => {
                match next {
                    AdaptiveRedactionSegmentStatus::Revoked | AdaptiveRedactionSegmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRedactionSegmentStatus::Revoked => {
                return next == AdaptiveRedactionSegmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRedactionSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive redaction segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> AdaptiveRedactionSegment {
        AdaptiveRedactionSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            timezone: self.timezone.clone(),
            pinned: self.pinned,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive redaction segment in status requested.
pub fn new_adaptive_redaction_segment(code: &str, name: &str) -> AdaptiveRedactionSegment {
    AdaptiveRedactionSegment {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        timezone: "".to_string(),
        pinned: false,
        floor_value: 0.0,
        status: AdaptiveRedactionSegmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_adaptive_redaction_segment("code-363", "name-611");
        assert_eq!(item.status, AdaptiveRedactionSegmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_adaptive_redaction_segment("code-363", "name-611");
        if item.move_to(AdaptiveRedactionSegmentStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRedactionSegmentStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_adaptive_redaction_segment("code-363", "name-611");
        item.status = AdaptiveRedactionSegmentStatus::Expired;
        assert!(
            item.move_to(AdaptiveRedactionSegmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
