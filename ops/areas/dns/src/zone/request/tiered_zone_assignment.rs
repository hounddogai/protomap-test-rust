use super::*;

use acme_kit::text;

/// Where a tiered zone assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredZoneAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredZoneAssignmentStatus {
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

impl std::fmt::Display for TieredZoneAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered zone assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneAssignment {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub burst: i64,
    pub priority: i64,
    pub ttl_seconds: i64,
    pub status: TieredZoneAssignmentStatus,
}

impl TieredZoneAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredZoneAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredZoneAssignmentStatus) -> bool {
        match self.status {
            TieredZoneAssignmentStatus::Draft => {
                return next == TieredZoneAssignmentStatus::Active;
            }
            TieredZoneAssignmentStatus::Pending => {
                return next == TieredZoneAssignmentStatus::Active || next == TieredZoneAssignmentStatus::Closed;
            }
            TieredZoneAssignmentStatus::Active => {
                match next {
                    TieredZoneAssignmentStatus::Suspended | TieredZoneAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredZoneAssignmentStatus::Suspended => {
                return next == TieredZoneAssignmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredZoneAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered zone assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered zone assignment", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredZoneAssignment {
        TieredZoneAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            burst: self.burst,
            priority: self.priority,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered zone assignment in status draft.
pub fn new_tiered_zone_assignment(code: &str, name: &str) -> TieredZoneAssignment {
    TieredZoneAssignment {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        burst: 0,
        priority: 0,
        ttl_seconds: 0,
        status: TieredZoneAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_zone_assignment("code-443", "name-646");
        assert_eq!(item.status, TieredZoneAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_tiered_zone_assignment("code-443", "name-646");
        if item.move_to(TieredZoneAssignmentStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredZoneAssignmentStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_zone_assignment("code-443", "name-646");
        item.status = TieredZoneAssignmentStatus::Closed;
        assert!(
            item.move_to(TieredZoneAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
