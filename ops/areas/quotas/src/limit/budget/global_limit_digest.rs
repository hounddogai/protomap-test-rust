use super::*;

use acme_kit::text;

/// Where a global limit digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalLimitDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalLimitDigestStatus {
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

impl std::fmt::Display for GlobalLimitDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global limit digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDigest {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub notes: String,
    pub principal: String,
    pub margin: f64,
    pub status: GlobalLimitDigestStatus,
}

impl GlobalLimitDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalLimitDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalLimitDigestStatus) -> bool {
        match self.status {
            GlobalLimitDigestStatus::Draft => {
                match next {
                    GlobalLimitDigestStatus::Suspended | GlobalLimitDigestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalLimitDigestStatus::Pending => {
                return next == GlobalLimitDigestStatus::Active || next == GlobalLimitDigestStatus::Suspended;
            }
            GlobalLimitDigestStatus::Active => {
                match next {
                    GlobalLimitDigestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalLimitDigestStatus::Suspended => {
                return next == GlobalLimitDigestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalLimitDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global limit digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> GlobalLimitDigest {
        GlobalLimitDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            notes: self.notes.clone(),
            principal: self.principal.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global limit digest in status draft.
pub fn new_global_limit_digest(code: &str, title: &str) -> GlobalLimitDigest {
    GlobalLimitDigest {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        notes: "".to_string(),
        principal: "".to_string(),
        margin: 0.0,
        status: GlobalLimitDigestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_limit_digest("code-16", "title-578");
        assert_eq!(item.status, GlobalLimitDigestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_global_limit_digest("code-16", "title-578");
        if item.move_to(GlobalLimitDigestStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalLimitDigestStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_limit_digest("code-16", "title-578");
        item.status = GlobalLimitDigestStatus::Closed;
        assert!(
            item.move_to(GlobalLimitDigestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
