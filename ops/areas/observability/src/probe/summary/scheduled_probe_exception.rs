use super::*;

use acme_kit::text;

/// Where a scheduled probe exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledProbeExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledProbeExceptionStatus {
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

impl std::fmt::Display for ScheduledProbeExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled probe exception as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeException {
    pub code: String,
    pub label: String,
    pub title: String,
    pub contact_email: String,
    pub expires_at: i64,
    pub created_at: i64,
    pub key_id: String,
    pub fingerprint: String,
    pub status: ScheduledProbeExceptionStatus,
}

impl ScheduledProbeException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledProbeExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledProbeExceptionStatus) -> bool {
        match self.status {
            ScheduledProbeExceptionStatus::Open => {
                match next {
                    ScheduledProbeExceptionStatus::Reopened | ScheduledProbeExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledProbeExceptionStatus::InReview => {
                return next == ScheduledProbeExceptionStatus::Resolved
                    || next == ScheduledProbeExceptionStatus::Reopened;
            }
            ScheduledProbeExceptionStatus::Resolved => {
                match next {
                    ScheduledProbeExceptionStatus::Reopened | ScheduledProbeExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledProbeExceptionStatus::Reopened => {
                return next == ScheduledProbeExceptionStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledProbeExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled probe exception {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.expires_at as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledProbeException {
        ScheduledProbeException {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            expires_at: self.expires_at,
            created_at: self.created_at,
            key_id: self.key_id.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled probe exception in status open.
pub fn new_scheduled_probe_exception(code: &str, label: &str) -> ScheduledProbeException {
    ScheduledProbeException {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        contact_email: "".to_string(),
        expires_at: 0,
        created_at: 0,
        key_id: "".to_string(),
        fingerprint: "".to_string(),
        status: ScheduledProbeExceptionStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_probe_exception("code-146", "label-228");
        assert_eq!(item.status, ScheduledProbeExceptionStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_scheduled_probe_exception("code-146", "label-228");
        if item.move_to(ScheduledProbeExceptionStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledProbeExceptionStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_probe_exception("code-146", "label-228");
        item.status = ScheduledProbeExceptionStatus::Archived;
        assert!(
            item.move_to(ScheduledProbeExceptionStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
