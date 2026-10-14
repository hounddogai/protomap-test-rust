use super::*;

use acme_kit::text;

/// Where a timeline threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineThresholdStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TimelineThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TimelineThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineThreshold {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub discount: f64,
    pub expires_at: i64,
    pub status: TimelineThresholdStatus,
}

impl TimelineThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineThresholdStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineThresholdStatus) -> bool {
        match self.status {
            TimelineThresholdStatus::Proposed => {
                match next {
                    TimelineThresholdStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineThresholdStatus::Approved => {
                match next {
                    TimelineThresholdStatus::Rejected | TimelineThresholdStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineThresholdStatus::Rejected => {
                return next == TimelineThresholdStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline threshold {} cannot move from {} to {}",
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

    pub fn weighted_load(&self) -> f64 {
        ((self.port as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineThreshold {
        TimelineThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            discount: self.discount,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a timeline threshold in status proposed.
pub fn new_timeline_threshold(code: &str, title: &str) -> TimelineThreshold {
    TimelineThreshold {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        discount: 0.0,
        expires_at: 0,
        status: TimelineThresholdStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_timeline_threshold("code-761", "title-78");
        assert_eq!(item.status, TimelineThresholdStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_timeline_threshold("code-761", "title-78");
        if item.move_to(TimelineThresholdStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineThresholdStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_timeline_threshold("code-761", "title-78");
        item.status = TimelineThresholdStatus::Withdrawn;
        assert!(
            item.move_to(TimelineThresholdStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
