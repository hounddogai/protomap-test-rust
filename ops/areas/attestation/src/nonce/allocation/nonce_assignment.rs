use super::*;

use acme_kit::text;

/// Where a nonce assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceAssignmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl NonceAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for NonceAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a nonce assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceAssignment {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub country: String,
    pub version: i64,
    pub namespace: String,
    pub cipher: String,
    pub created_at: i64,
    pub score: f64,
    pub floor_value: f64,
    pub status: NonceAssignmentStatus,
}

impl NonceAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceAssignmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceAssignmentStatus) -> bool {
        match self.status {
            NonceAssignmentStatus::Requested => {
                match next {
                    NonceAssignmentStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceAssignmentStatus::Granted => {
                match next {
                    NonceAssignmentStatus::Revoked | NonceAssignmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceAssignmentStatus::Revoked => {
                match next {
                    NonceAssignmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: NonceAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "nonce assignment", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.floor_value + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceAssignment {
        NonceAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            country: self.country.clone(),
            version: self.version,
            namespace: self.namespace.clone(),
            cipher: self.cipher.clone(),
            created_at: self.created_at,
            score: self.score,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a nonce assignment in status requested.
pub fn new_nonce_assignment(code: &str, label: &str) -> NonceAssignment {
    NonceAssignment {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        country: "".to_string(),
        version: 0,
        namespace: "".to_string(),
        cipher: "".to_string(),
        created_at: 0,
        score: 0.0,
        floor_value: 0.0,
        status: NonceAssignmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_nonce_assignment("code-334", "label-38");
        assert_eq!(item.status, NonceAssignmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_nonce_assignment("code-334", "label-38");
        if item.move_to(NonceAssignmentStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceAssignmentStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_nonce_assignment("code-334", "label-38");
        item.status = NonceAssignmentStatus::Expired;
        assert!(
            item.move_to(NonceAssignmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
