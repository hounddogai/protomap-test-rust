use super::*;

use acme_kit::text;

/// Where a allowlist mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AllowlistMappingStatus {
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

impl std::fmt::Display for AllowlistMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist mapping as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMapping {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub description: String,
    pub channel: String,
    pub fingerprint: String,
    pub quantity: i64,
    pub status: AllowlistMappingStatus,
}

impl AllowlistMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistMappingStatus) -> bool {
        match self.status {
            AllowlistMappingStatus::Planned => {
                match next {
                    AllowlistMappingStatus::Completed | AllowlistMappingStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistMappingStatus::Scheduled => {
                return next == AllowlistMappingStatus::InProgress || next == AllowlistMappingStatus::Cancelled;
            }
            AllowlistMappingStatus::InProgress => {
                return next == AllowlistMappingStatus::Completed || next == AllowlistMappingStatus::Cancelled;
            }
            AllowlistMappingStatus::Completed => {
                return next == AllowlistMappingStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "allowlist mapping", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> AllowlistMapping {
        AllowlistMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            description: self.description.clone(),
            channel: self.channel.clone(),
            fingerprint: self.fingerprint.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a allowlist mapping in status planned.
pub fn new_allowlist_mapping(code: &str, name: &str) -> AllowlistMapping {
    AllowlistMapping {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        description: "".to_string(),
        channel: "".to_string(),
        fingerprint: "".to_string(),
        quantity: 0,
        status: AllowlistMappingStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_allowlist_mapping("code-823", "name-13");
        assert_eq!(item.status, AllowlistMappingStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_allowlist_mapping("code-823", "name-13");
        if item.move_to(AllowlistMappingStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistMappingStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_allowlist_mapping("code-823", "name-13");
        item.status = AllowlistMappingStatus::Cancelled;
        assert!(
            item.move_to(AllowlistMappingStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
