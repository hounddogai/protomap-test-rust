use super::*;

use acme_kit::text;

/// Where a scheduled credential segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledCredentialSegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledCredentialSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ScheduledCredentialSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled credential segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCredentialSegment {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub floor_value: f64,
    pub principal: String,
    pub key_id: String,
    pub version: i64,
    pub burst: i64,
    pub cipher: String,
    pub status: ScheduledCredentialSegmentStatus,
}

impl ScheduledCredentialSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledCredentialSegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledCredentialSegmentStatus) -> bool {
        match self.status {
            ScheduledCredentialSegmentStatus::Open => {
                match next {
                    ScheduledCredentialSegmentStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledCredentialSegmentStatus::InReview => {
                match next {
                    ScheduledCredentialSegmentStatus::Resolved | ScheduledCredentialSegmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledCredentialSegmentStatus::Resolved => {
                match next {
                    ScheduledCredentialSegmentStatus::Reopened | ScheduledCredentialSegmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledCredentialSegmentStatus::Reopened => {
                match next {
                    ScheduledCredentialSegmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: ScheduledCredentialSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled credential segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled credential segment", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.version as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledCredentialSegment {
        ScheduledCredentialSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            floor_value: self.floor_value,
            principal: self.principal.clone(),
            key_id: self.key_id.clone(),
            version: self.version,
            burst: self.burst,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled credential segment in status open.
pub fn new_scheduled_credential_segment(code: &str, title: &str) -> ScheduledCredentialSegment {
    ScheduledCredentialSegment {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        floor_value: 0.0,
        principal: "".to_string(),
        key_id: "".to_string(),
        version: 0,
        burst: 0,
        cipher: "".to_string(),
        status: ScheduledCredentialSegmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_credential_segment("code-502", "title-356");
        assert_eq!(item.status, ScheduledCredentialSegmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_scheduled_credential_segment("code-502", "title-356");
        if item.move_to(ScheduledCredentialSegmentStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledCredentialSegmentStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_credential_segment("code-502", "title-356");
        item.status = ScheduledCredentialSegmentStatus::Archived;
        assert!(
            item.move_to(ScheduledCredentialSegmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
