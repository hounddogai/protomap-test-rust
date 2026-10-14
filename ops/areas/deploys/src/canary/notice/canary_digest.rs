use super::*;

use acme_kit::text;

/// Where a canary digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryDigestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CanaryDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CanaryDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDigest {
    pub code: String,
    pub label: String,
    pub category: String,
    pub external_ref: String,
    pub sequence: i64,
    pub key_id: String,
    pub region: String,
    pub host: String,
    pub status: CanaryDigestStatus,
}

impl CanaryDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryDigestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryDigestStatus) -> bool {
        match self.status {
            CanaryDigestStatus::Requested => {
                return next == CanaryDigestStatus::Granted || next == CanaryDigestStatus::Expired;
            }
            CanaryDigestStatus::Granted => {
                match next {
                    CanaryDigestStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryDigestStatus::Revoked => {
                match next {
                    CanaryDigestStatus::Expired => {
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

    pub fn move_to(&mut self, next: CanaryDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> CanaryDigest {
        CanaryDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            external_ref: self.external_ref.clone(),
            sequence: self.sequence,
            key_id: self.key_id.clone(),
            region: self.region.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a canary digest in status requested.
pub fn new_canary_digest(code: &str, label: &str) -> CanaryDigest {
    CanaryDigest {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        external_ref: "".to_string(),
        sequence: 0,
        key_id: "".to_string(),
        region: "".to_string(),
        host: "".to_string(),
        status: CanaryDigestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_canary_digest("code-650", "label-387");
        assert_eq!(item.status, CanaryDigestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_canary_digest("code-650", "label-387");
        if item.move_to(CanaryDigestStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryDigestStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_canary_digest("code-650", "label-387");
        item.status = CanaryDigestStatus::Expired;
        assert!(
            item.move_to(CanaryDigestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
