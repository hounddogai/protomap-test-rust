use super::*;

use acme_kit::text;

/// Where a segment slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SegmentSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SegmentSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One segment slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSlot {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub window_days: i64,
    pub host: String,
    pub issuer: String,
    pub description: String,
    pub label: String,
    pub status: SegmentSlotStatus,
}

impl SegmentSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentSlotStatus) -> bool {
        match self.status {
            SegmentSlotStatus::Requested => {
                match next {
                    SegmentSlotStatus::Granted | SegmentSlotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentSlotStatus::Granted => {
                match next {
                    SegmentSlotStatus::Revoked | SegmentSlotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentSlotStatus::Revoked => {
                return next == SegmentSlotStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.expires_at as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentSlot {
        SegmentSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            window_days: self.window_days,
            host: self.host.clone(),
            issuer: self.issuer.clone(),
            description: self.description.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a segment slot in status requested.
pub fn new_segment_slot(code: &str, title: &str) -> SegmentSlot {
    SegmentSlot {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        window_days: 0,
        host: "".to_string(),
        issuer: "".to_string(),
        description: "".to_string(),
        label: "".to_string(),
        status: SegmentSlotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_segment_slot("code-828", "title-158");
        assert_eq!(item.status, SegmentSlotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_segment_slot("code-828", "title-158");
        if item.move_to(SegmentSlotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentSlotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_segment_slot("code-828", "title-158");
        item.status = SegmentSlotStatus::Expired;
        assert!(
            item.move_to(SegmentSlotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
