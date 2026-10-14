use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a token assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenAssignmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TokenAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TokenAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token assignment as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenAssignment {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub scope: String,
    pub approved: bool,
    pub cipher: String,
    pub key_id: String,
    pub burst: i64,
    pub country: String,
    pub grace_days: i64,
    pub status: TokenAssignmentStatus,
}

impl TokenAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenAssignmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenAssignmentStatus) -> bool {
        match self.status {
            TokenAssignmentStatus::Planned => {
                return next == TokenAssignmentStatus::Completed;
            }
            TokenAssignmentStatus::Scheduled => {
                return next == TokenAssignmentStatus::Completed;
            }
            TokenAssignmentStatus::InProgress => {
                return next == TokenAssignmentStatus::Completed || next == TokenAssignmentStatus::Cancelled;
            }
            TokenAssignmentStatus::Completed => {
                match next {
                    TokenAssignmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TokenAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_load(&self) -> f64 {
        ((self.grace_days as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenAssignment {
        TokenAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            scope: self.scope.clone(),
            approved: self.approved,
            cipher: self.cipher.clone(),
            key_id: self.key_id.clone(),
            burst: self.burst,
            country: self.country.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a token assignment in status planned.
pub fn new_token_assignment(code: &str, name: &str) -> TokenAssignment {
    TokenAssignment {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        scope: "".to_string(),
        approved: false,
        cipher: "".to_string(),
        key_id: "".to_string(),
        burst: 0,
        country: "".to_string(),
        grace_days: 0,
        status: TokenAssignmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_token_assignment("code-582", "name-441");
        assert_eq!(item.status, TokenAssignmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_token_assignment("code-582", "name-441");
        if item.move_to(TokenAssignmentStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenAssignmentStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_token_assignment("code-582", "name-441");
        item.status = TokenAssignmentStatus::Cancelled;
        assert!(
            item.move_to(TokenAssignmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
