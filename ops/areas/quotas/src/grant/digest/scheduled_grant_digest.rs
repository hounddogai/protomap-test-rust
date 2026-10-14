use super::*;

use acme_kit::text;

/// Where a scheduled grant digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledGrantDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledGrantDigestStatus {
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

impl std::fmt::Display for ScheduledGrantDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled grant digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantDigest {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub limit: i64,
    pub revision: i64,
    pub notes: String,
    pub contact_email: String,
    pub channel: String,
    pub namespace: String,
    pub status: ScheduledGrantDigestStatus,
}

impl ScheduledGrantDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledGrantDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledGrantDigestStatus) -> bool {
        match self.status {
            ScheduledGrantDigestStatus::Draft => {
                return next == ScheduledGrantDigestStatus::Suspended;
            }
            ScheduledGrantDigestStatus::Pending => {
                return next == ScheduledGrantDigestStatus::Active || next == ScheduledGrantDigestStatus::Suspended;
            }
            ScheduledGrantDigestStatus::Active => {
                match next {
                    ScheduledGrantDigestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledGrantDigestStatus::Suspended => {
                match next {
                    ScheduledGrantDigestStatus::Closed => {
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

    pub fn move_to(&mut self, next: ScheduledGrantDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled grant digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledGrantDigest {
        ScheduledGrantDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            limit: self.limit,
            revision: self.revision,
            notes: self.notes.clone(),
            contact_email: self.contact_email.clone(),
            channel: self.channel.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a scheduled grant digest in status draft.
pub fn new_scheduled_grant_digest(code: &str, name: &str) -> ScheduledGrantDigest {
    ScheduledGrantDigest {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        limit: 0,
        revision: 0,
        notes: "".to_string(),
        contact_email: "".to_string(),
        channel: "".to_string(),
        namespace: "".to_string(),
        status: ScheduledGrantDigestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_grant_digest("code-627", "name-752");
        assert_eq!(item.status, ScheduledGrantDigestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_scheduled_grant_digest("code-627", "name-752");
        if item.move_to(ScheduledGrantDigestStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledGrantDigestStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_grant_digest("code-627", "name-752");
        item.status = ScheduledGrantDigestStatus::Closed;
        assert!(
            item.move_to(ScheduledGrantDigestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
