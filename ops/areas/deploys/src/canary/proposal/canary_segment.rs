use super::*;

use acme_kit::text;

/// Where a canary segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanarySegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CanarySegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for CanarySegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CanarySegment {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub ratio: f64,
    pub revision: i64,
    pub notes: String,
    pub audience: String,
    pub host: String,
    pub attempts: i64,
    pub rotation_days: i64,
    pub status: CanarySegmentStatus,
}

impl CanarySegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanarySegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanarySegmentStatus) -> bool {
        match self.status {
            CanarySegmentStatus::Draft => {
                match next {
                    CanarySegmentStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanarySegmentStatus::Pending => {
                match next {
                    CanarySegmentStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanarySegmentStatus::Active => {
                return next == CanarySegmentStatus::Closed;
            }
            CanarySegmentStatus::Suspended => {
                return next == CanarySegmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanarySegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "canary segment", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.ends_at as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanarySegment {
        CanarySegment {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            ratio: self.ratio,
            revision: self.revision,
            notes: self.notes.clone(),
            audience: self.audience.clone(),
            host: self.host.clone(),
            attempts: self.attempts,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a canary segment in status draft.
pub fn new_canary_segment(code: &str, title: &str) -> CanarySegment {
    CanarySegment {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        ratio: 0.0,
        revision: 0,
        notes: "".to_string(),
        audience: "".to_string(),
        host: "".to_string(),
        attempts: 0,
        rotation_days: 0,
        status: CanarySegmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_canary_segment("code-709", "title-612");
        assert_eq!(item.status, CanarySegmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_canary_segment("code-709", "title-612");
        if item.move_to(CanarySegmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanarySegmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_canary_segment("code-709", "title-612");
        item.status = CanarySegmentStatus::Closed;
        assert!(
            item.move_to(CanarySegmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
