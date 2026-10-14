use super::*;

use acme_kit::text;

/// Where a ttl policy exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TtlPolicyExceptionStatus {
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

impl std::fmt::Display for TtlPolicyExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyException {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub floor_value: f64,
    pub expires_at: i64,
    pub ratio: f64,
    pub maximum: f64,
    pub tags: Vec<String>,
    pub issuer: String,
    pub status: TtlPolicyExceptionStatus,
}

impl TtlPolicyException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyExceptionStatus) -> bool {
        match self.status {
            TtlPolicyExceptionStatus::Draft => {
                return next == TtlPolicyExceptionStatus::Pending || next == TtlPolicyExceptionStatus::Suspended;
            }
            TtlPolicyExceptionStatus::Pending => {
                match next {
                    TtlPolicyExceptionStatus::Active | TtlPolicyExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyExceptionStatus::Active => {
                match next {
                    TtlPolicyExceptionStatus::Suspended | TtlPolicyExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyExceptionStatus::Suspended => {
                match next {
                    TtlPolicyExceptionStatus::Closed => {
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

    pub fn move_to(&mut self, next: TtlPolicyExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy exception {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_exposure(&self) -> f64 {
        (self.maximum * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyException {
        TtlPolicyException {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            floor_value: self.floor_value,
            expires_at: self.expires_at,
            ratio: self.ratio,
            maximum: self.maximum,
            tags: self.tags.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a ttl policy exception in status draft.
pub fn new_ttl_policy_exception(code: &str, name: &str) -> TtlPolicyException {
    TtlPolicyException {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        floor_value: 0.0,
        expires_at: 0,
        ratio: 0.0,
        maximum: 0.0,
        tags: Vec::new(),
        issuer: "".to_string(),
        status: TtlPolicyExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_ttl_policy_exception("code-191", "name-964");
        assert_eq!(item.status, TtlPolicyExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_ttl_policy_exception("code-191", "name-964");
        if item.move_to(TtlPolicyExceptionStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyExceptionStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_ttl_policy_exception("code-191", "name-964");
        item.status = TtlPolicyExceptionStatus::Closed;
        assert!(
            item.move_to(TtlPolicyExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
