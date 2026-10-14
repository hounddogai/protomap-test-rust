use super::*;

use acme_kit::text;

/// Where a primary query assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryQueryAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryQueryAssignmentStatus {
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

impl std::fmt::Display for PrimaryQueryAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary query assignment as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQueryAssignment {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub timezone: String,
    pub updated_at: i64,
    pub created_at: i64,
    pub cipher: String,
    pub weight: f64,
    pub scope: String,
    pub checksum: String,
    pub status: PrimaryQueryAssignmentStatus,
}

impl PrimaryQueryAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryQueryAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryQueryAssignmentStatus) -> bool {
        match self.status {
            PrimaryQueryAssignmentStatus::Draft => {
                return next == PrimaryQueryAssignmentStatus::Pending || next == PrimaryQueryAssignmentStatus::Active;
            }
            PrimaryQueryAssignmentStatus::Pending => {
                match next {
                    PrimaryQueryAssignmentStatus::Active | PrimaryQueryAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryQueryAssignmentStatus::Active => {
                match next {
                    PrimaryQueryAssignmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryQueryAssignmentStatus::Suspended => {
                return next == PrimaryQueryAssignmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryQueryAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary query assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.weight + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryQueryAssignment {
        PrimaryQueryAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            timezone: self.timezone.clone(),
            updated_at: self.updated_at,
            created_at: self.created_at,
            cipher: self.cipher.clone(),
            weight: self.weight,
            scope: self.scope.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a primary query assignment in status draft.
pub fn new_primary_query_assignment(code: &str, name: &str) -> PrimaryQueryAssignment {
    PrimaryQueryAssignment {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        timezone: "".to_string(),
        updated_at: 0,
        created_at: 0,
        cipher: "".to_string(),
        weight: 0.0,
        scope: "".to_string(),
        checksum: "".to_string(),
        status: PrimaryQueryAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_query_assignment("code-765", "name-318");
        assert_eq!(item.status, PrimaryQueryAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_primary_query_assignment("code-765", "name-318");
        if item.move_to(PrimaryQueryAssignmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryQueryAssignmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_query_assignment("code-765", "name-318");
        item.status = PrimaryQueryAssignmentStatus::Closed;
        assert!(
            item.move_to(PrimaryQueryAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
