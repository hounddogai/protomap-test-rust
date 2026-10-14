use super::*;

use acme_kit::text;

/// Where a pending token digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingTokenDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingTokenDigestStatus {
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

impl std::fmt::Display for PendingTokenDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending token digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTokenDigest {
    pub code: String,
    pub name: String,
    pub source: String,
    pub phone: String,
    pub revision: i64,
    pub status: PendingTokenDigestStatus,
}

impl PendingTokenDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingTokenDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingTokenDigestStatus) -> bool {
        match self.status {
            PendingTokenDigestStatus::Draft => {
                return next == PendingTokenDigestStatus::Active;
            }
            PendingTokenDigestStatus::Pending => {
                return next == PendingTokenDigestStatus::Suspended || next == PendingTokenDigestStatus::Closed;
            }
            PendingTokenDigestStatus::Active => {
                return next == PendingTokenDigestStatus::Closed;
            }
            PendingTokenDigestStatus::Suspended => {
                return next == PendingTokenDigestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingTokenDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending token digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending token digest", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> PendingTokenDigest {
        PendingTokenDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            phone: self.phone.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending token digest in status draft.
pub fn new_pending_token_digest(code: &str, name: &str) -> PendingTokenDigest {
    PendingTokenDigest {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        phone: "".to_string(),
        revision: 0,
        status: PendingTokenDigestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_token_digest("code-394", "name-467");
        assert_eq!(item.status, PendingTokenDigestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_pending_token_digest("code-394", "name-467");
        if item.move_to(PendingTokenDigestStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingTokenDigestStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_token_digest("code-394", "name-467");
        item.status = PendingTokenDigestStatus::Closed;
        assert!(
            item.move_to(PendingTokenDigestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
