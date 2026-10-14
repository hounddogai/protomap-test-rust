use super::*;

use acme_kit::text;

/// Where a credential schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CredentialScheduleStatus {
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

impl std::fmt::Display for CredentialScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSchedule {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub summary: String,
    pub threshold: f64,
    pub ttl_seconds: i64,
    pub audience: String,
    pub status: CredentialScheduleStatus,
}

impl CredentialSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialScheduleStatus) -> bool {
        match self.status {
            CredentialScheduleStatus::Open => {
                return next == CredentialScheduleStatus::Archived;
            }
            CredentialScheduleStatus::InReview => {
                return next == CredentialScheduleStatus::Resolved || next == CredentialScheduleStatus::Reopened;
            }
            CredentialScheduleStatus::Resolved => {
                match next {
                    CredentialScheduleStatus::Reopened | CredentialScheduleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialScheduleStatus::Reopened => {
                match next {
                    CredentialScheduleStatus::Archived => {
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

    pub fn move_to(&mut self, next: CredentialScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "credential schedule", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialSchedule {
        CredentialSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            summary: self.summary.clone(),
            threshold: self.threshold,
            ttl_seconds: self.ttl_seconds,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a credential schedule in status open.
pub fn new_credential_schedule(code: &str, title: &str) -> CredentialSchedule {
    CredentialSchedule {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        summary: "".to_string(),
        threshold: 0.0,
        ttl_seconds: 0,
        audience: "".to_string(),
        status: CredentialScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_credential_schedule("code-132", "title-876");
        assert_eq!(item.status, CredentialScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_credential_schedule("code-132", "title-876");
        if item.move_to(CredentialScheduleStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialScheduleStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_credential_schedule("code-132", "title-876");
        item.status = CredentialScheduleStatus::Archived;
        assert!(
            item.move_to(CredentialScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
