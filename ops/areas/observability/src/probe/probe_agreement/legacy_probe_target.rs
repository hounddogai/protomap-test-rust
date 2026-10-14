use super::*;

use acme_kit::text;

/// Where a legacy probe target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyProbeTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyProbeTargetStatus {
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

impl std::fmt::Display for LegacyProbeTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy probe target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeTarget {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub notes: String,
    pub capacity: i64,
    pub created_at: i64,
    pub external_ref: String,
    pub status: LegacyProbeTargetStatus,
}

impl LegacyProbeTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyProbeTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyProbeTargetStatus) -> bool {
        match self.status {
            LegacyProbeTargetStatus::Planned => {
                match next {
                    LegacyProbeTargetStatus::Scheduled | LegacyProbeTargetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyProbeTargetStatus::Scheduled => {
                return next == LegacyProbeTargetStatus::InProgress || next == LegacyProbeTargetStatus::Cancelled;
            }
            LegacyProbeTargetStatus::InProgress => {
                match next {
                    LegacyProbeTargetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyProbeTargetStatus::Completed => {
                match next {
                    LegacyProbeTargetStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LegacyProbeTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy probe target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.created_at as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyProbeTarget {
        LegacyProbeTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            notes: self.notes.clone(),
            capacity: self.capacity,
            created_at: self.created_at,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy probe target in status planned.
pub fn new_legacy_probe_target(code: &str, name: &str) -> LegacyProbeTarget {
    LegacyProbeTarget {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        notes: "".to_string(),
        capacity: 0,
        created_at: 0,
        external_ref: "".to_string(),
        status: LegacyProbeTargetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_legacy_probe_target("code-742", "name-460");
        assert_eq!(item.status, LegacyProbeTargetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_legacy_probe_target("code-742", "name-460");
        if item.move_to(LegacyProbeTargetStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyProbeTargetStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_legacy_probe_target("code-742", "name-460");
        item.status = LegacyProbeTargetStatus::Cancelled;
        assert!(
            item.move_to(LegacyProbeTargetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
