use super::*;

use acme_kit::text;

/// Where a pending zone assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingZoneAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingZoneAssignmentStatus {
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

impl std::fmt::Display for PendingZoneAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending zone assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingZoneAssignment {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub category: String,
    pub score: f64,
    pub cluster: String,
    pub ceiling: f64,
    pub cost: f64,
    pub starts_at: i64,
    pub notes: String,
    pub status: PendingZoneAssignmentStatus,
}

impl PendingZoneAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingZoneAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingZoneAssignmentStatus) -> bool {
        match self.status {
            PendingZoneAssignmentStatus::Draft => {
                match next {
                    PendingZoneAssignmentStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingZoneAssignmentStatus::Pending => {
                match next {
                    PendingZoneAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingZoneAssignmentStatus::Active => {
                return next == PendingZoneAssignmentStatus::Suspended || next == PendingZoneAssignmentStatus::Closed;
            }
            PendingZoneAssignmentStatus::Suspended => {
                match next {
                    PendingZoneAssignmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: PendingZoneAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending zone assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.cost - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingZoneAssignment {
        PendingZoneAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            category: self.category.clone(),
            score: self.score,
            cluster: self.cluster.clone(),
            ceiling: self.ceiling,
            cost: self.cost,
            starts_at: self.starts_at,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a pending zone assignment in status draft.
pub fn new_pending_zone_assignment(code: &str, label: &str) -> PendingZoneAssignment {
    PendingZoneAssignment {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        category: "".to_string(),
        score: 0.0,
        cluster: "".to_string(),
        ceiling: 0.0,
        cost: 0.0,
        starts_at: 0,
        notes: "".to_string(),
        status: PendingZoneAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_zone_assignment("code-10", "label-76");
        assert_eq!(item.status, PendingZoneAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_pending_zone_assignment("code-10", "label-76");
        if item.move_to(PendingZoneAssignmentStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingZoneAssignmentStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_zone_assignment("code-10", "label-76");
        item.status = PendingZoneAssignmentStatus::Closed;
        assert!(
            item.move_to(PendingZoneAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
