use super::*;

use acme_kit::text;

/// Where a segment window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentWindowStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SegmentWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SegmentWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentWindow {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub pinned: bool,
    pub cost: f64,
    pub namespace: String,
    pub ttl_seconds: i64,
    pub duration_minutes: i64,
    pub burst: i64,
    pub status: SegmentWindowStatus,
}

impl SegmentWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentWindowStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentWindowStatus) -> bool {
        match self.status {
            SegmentWindowStatus::Requested => {
                return next == SegmentWindowStatus::Granted;
            }
            SegmentWindowStatus::Granted => {
                return next == SegmentWindowStatus::Revoked || next == SegmentWindowStatus::Expired;
            }
            SegmentWindowStatus::Revoked => {
                match next {
                    SegmentWindowStatus::Expired => {
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

    pub fn move_to(&mut self, next: SegmentWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment window {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.cost - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentWindow {
        SegmentWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            pinned: self.pinned,
            cost: self.cost,
            namespace: self.namespace.clone(),
            ttl_seconds: self.ttl_seconds,
            duration_minutes: self.duration_minutes,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a segment window in status requested.
pub fn new_segment_window(code: &str, title: &str) -> SegmentWindow {
    SegmentWindow {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        pinned: false,
        cost: 0.0,
        namespace: "".to_string(),
        ttl_seconds: 0,
        duration_minutes: 0,
        burst: 0,
        status: SegmentWindowStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_segment_window("code-995", "title-671");
        assert_eq!(item.status, SegmentWindowStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_segment_window("code-995", "title-671");
        if item.move_to(SegmentWindowStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentWindowStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_segment_window("code-995", "title-671");
        item.status = SegmentWindowStatus::Expired;
        assert!(
            item.move_to(SegmentWindowStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
