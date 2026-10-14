use super::*;

use acme_kit::text;

/// Where a weighted retention assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRetentionAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedRetentionAssignmentStatus {
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

impl std::fmt::Display for WeightedRetentionAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted retention assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionAssignment {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub updated_at: i64,
    pub scope: String,
    pub cost: f64,
    pub status: WeightedRetentionAssignmentStatus,
}

impl WeightedRetentionAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRetentionAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRetentionAssignmentStatus) -> bool {
        match self.status {
            WeightedRetentionAssignmentStatus::Draft => {
                match next {
                    WeightedRetentionAssignmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRetentionAssignmentStatus::Pending => {
                return next == WeightedRetentionAssignmentStatus::Suspended;
            }
            WeightedRetentionAssignmentStatus::Active => {
                match next {
                    WeightedRetentionAssignmentStatus::Suspended | WeightedRetentionAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRetentionAssignmentStatus::Suspended => {
                match next {
                    WeightedRetentionAssignmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: WeightedRetentionAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted retention assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.updated_at as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRetentionAssignment {
        WeightedRetentionAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            updated_at: self.updated_at,
            scope: self.scope.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a weighted retention assignment in status draft.
pub fn new_weighted_retention_assignment(code: &str, name: &str) -> WeightedRetentionAssignment {
    WeightedRetentionAssignment {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        updated_at: 0,
        scope: "".to_string(),
        cost: 0.0,
        status: WeightedRetentionAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_weighted_retention_assignment("code-445", "name-123");
        assert_eq!(item.status, WeightedRetentionAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_weighted_retention_assignment("code-445", "name-123");
        if item.move_to(WeightedRetentionAssignmentStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRetentionAssignmentStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_weighted_retention_assignment("code-445", "name-123");
        item.status = WeightedRetentionAssignmentStatus::Closed;
        assert!(
            item.move_to(WeightedRetentionAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
