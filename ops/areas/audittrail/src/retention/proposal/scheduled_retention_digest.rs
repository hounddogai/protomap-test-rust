use super::*;

use acme_kit::text;

/// Where a scheduled retention digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRetentionDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledRetentionDigestStatus {
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

impl std::fmt::Display for ScheduledRetentionDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled retention digest as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionDigest {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub notes: String,
    pub description: String,
    pub quantity: i64,
    pub status: ScheduledRetentionDigestStatus,
}

impl ScheduledRetentionDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRetentionDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRetentionDigestStatus) -> bool {
        match self.status {
            ScheduledRetentionDigestStatus::Draft => {
                return next == ScheduledRetentionDigestStatus::Active
                    || next == ScheduledRetentionDigestStatus::Closed;
            }
            ScheduledRetentionDigestStatus::Pending => {
                return next == ScheduledRetentionDigestStatus::Active
                    || next == ScheduledRetentionDigestStatus::Suspended;
            }
            ScheduledRetentionDigestStatus::Active => {
                match next {
                    ScheduledRetentionDigestStatus::Suspended | ScheduledRetentionDigestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRetentionDigestStatus::Suspended => {
                return next == ScheduledRetentionDigestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledRetentionDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled retention digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled retention digest", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ScheduledRetentionDigest {
        ScheduledRetentionDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            notes: self.notes.clone(),
            description: self.description.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled retention digest in status draft.
pub fn new_scheduled_retention_digest(code: &str, title: &str) -> ScheduledRetentionDigest {
    ScheduledRetentionDigest {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        notes: "".to_string(),
        description: "".to_string(),
        quantity: 0,
        status: ScheduledRetentionDigestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_retention_digest("code-79", "title-581");
        assert_eq!(item.status, ScheduledRetentionDigestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_scheduled_retention_digest("code-79", "title-581");
        if item.move_to(ScheduledRetentionDigestStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledRetentionDigestStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_retention_digest("code-79", "title-581");
        item.status = ScheduledRetentionDigestStatus::Closed;
        assert!(
            item.move_to(ScheduledRetentionDigestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
