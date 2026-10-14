use super::*;

use acme_kit::text;

/// Where a scheduled policy digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledPolicyDigestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledPolicyDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledPolicyDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled policy digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPolicyDigest {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub tags: Vec<String>,
    pub grace_days: i64,
    pub archived: bool,
    pub active: bool,
    pub notes: String,
    pub audience: String,
    pub status: ScheduledPolicyDigestStatus,
}

impl ScheduledPolicyDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledPolicyDigestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledPolicyDigestStatus) -> bool {
        match self.status {
            ScheduledPolicyDigestStatus::Requested => {
                match next {
                    ScheduledPolicyDigestStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledPolicyDigestStatus::Granted => {
                return next == ScheduledPolicyDigestStatus::Revoked;
            }
            ScheduledPolicyDigestStatus::Revoked => {
                match next {
                    ScheduledPolicyDigestStatus::Expired => {
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

    pub fn move_to(&mut self, next: ScheduledPolicyDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled policy digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> ScheduledPolicyDigest {
        ScheduledPolicyDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            tags: self.tags.clone(),
            grace_days: self.grace_days,
            archived: self.archived,
            active: self.active,
            notes: self.notes.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a scheduled policy digest in status requested.
pub fn new_scheduled_policy_digest(code: &str, label: &str) -> ScheduledPolicyDigest {
    ScheduledPolicyDigest {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        tags: Vec::new(),
        grace_days: 0,
        archived: false,
        active: false,
        notes: "".to_string(),
        audience: "".to_string(),
        status: ScheduledPolicyDigestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_policy_digest("code-83", "label-234");
        assert_eq!(item.status, ScheduledPolicyDigestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_scheduled_policy_digest("code-83", "label-234");
        if item.move_to(ScheduledPolicyDigestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledPolicyDigestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_policy_digest("code-83", "label-234");
        item.status = ScheduledPolicyDigestStatus::Expired;
        assert!(
            item.move_to(ScheduledPolicyDigestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
