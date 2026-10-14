use super::*;

use acme_kit::text;

/// Where a pending segment preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSegmentPreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingSegmentPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingSegmentPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending segment preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPreference {
    pub code: String,
    pub title: String,
    pub host: String,
    pub margin: f64,
    pub maximum: f64,
    pub tenant_id: String,
    pub priority: i64,
    pub status: PendingSegmentPreferenceStatus,
}

impl PendingSegmentPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSegmentPreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSegmentPreferenceStatus) -> bool {
        match self.status {
            PendingSegmentPreferenceStatus::Requested => {
                match next {
                    PendingSegmentPreferenceStatus::Granted | PendingSegmentPreferenceStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSegmentPreferenceStatus::Granted => {
                match next {
                    PendingSegmentPreferenceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSegmentPreferenceStatus::Revoked => {
                return next == PendingSegmentPreferenceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingSegmentPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending segment preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.priority as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingSegmentPreference {
        PendingSegmentPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            margin: self.margin,
            maximum: self.maximum,
            tenant_id: self.tenant_id.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending segment preference in status requested.
pub fn new_pending_segment_preference(code: &str, title: &str) -> PendingSegmentPreference {
    PendingSegmentPreference {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        margin: 0.0,
        maximum: 0.0,
        tenant_id: "".to_string(),
        priority: 0,
        status: PendingSegmentPreferenceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_segment_preference("code-745", "title-934");
        assert_eq!(item.status, PendingSegmentPreferenceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_pending_segment_preference("code-745", "title-934");
        if item.move_to(PendingSegmentPreferenceStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingSegmentPreferenceStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_segment_preference("code-745", "title-934");
        item.status = PendingSegmentPreferenceStatus::Expired;
        assert!(
            item.move_to(PendingSegmentPreferenceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
