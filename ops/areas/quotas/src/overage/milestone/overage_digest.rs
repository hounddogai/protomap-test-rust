use super::*;

use acme_kit::text;

/// Where a overage digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl OverageDigestStatus {
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

impl std::fmt::Display for OverageDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One overage digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageDigest {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub tags: Vec<String>,
    pub contact_email: String,
    pub timezone: String,
    pub summary: String,
    pub key_id: String,
    pub amount: f64,
    pub status: OverageDigestStatus,
}

impl OverageDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageDigestStatus) -> bool {
        match self.status {
            OverageDigestStatus::Draft => {
                return next == OverageDigestStatus::Pending;
            }
            OverageDigestStatus::Pending => {
                match next {
                    OverageDigestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageDigestStatus::Active => {
                return next == OverageDigestStatus::Suspended;
            }
            OverageDigestStatus::Suspended => {
                return next == OverageDigestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
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
        self.amount
    }

    pub fn copy(&self) -> OverageDigest {
        OverageDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            tags: self.tags.clone(),
            contact_email: self.contact_email.clone(),
            timezone: self.timezone.clone(),
            summary: self.summary.clone(),
            key_id: self.key_id.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a overage digest in status draft.
pub fn new_overage_digest(code: &str, name: &str) -> OverageDigest {
    OverageDigest {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        tags: Vec::new(),
        contact_email: "".to_string(),
        timezone: "".to_string(),
        summary: "".to_string(),
        key_id: "".to_string(),
        amount: 0.0,
        status: OverageDigestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_overage_digest("code-569", "name-464");
        assert_eq!(item.status, OverageDigestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_overage_digest("code-569", "name-464");
        if item.move_to(OverageDigestStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageDigestStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_overage_digest("code-569", "name-464");
        item.status = OverageDigestStatus::Closed;
        assert!(
            item.move_to(OverageDigestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
