use super::*;

use acme_kit::text;

/// Where a capped isolation mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedIsolationMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedIsolationMappingStatus {
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

impl std::fmt::Display for CappedIsolationMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped isolation mapping as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationMapping {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub visible: bool,
    pub contact_email: String,
    pub cost: f64,
    pub tenant_id: String,
    pub attempts: i64,
    pub host: String,
    pub status: CappedIsolationMappingStatus,
}

impl CappedIsolationMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedIsolationMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedIsolationMappingStatus) -> bool {
        match self.status {
            CappedIsolationMappingStatus::Planned => {
                return next == CappedIsolationMappingStatus::Scheduled
                    || next == CappedIsolationMappingStatus::InProgress;
            }
            CappedIsolationMappingStatus::Scheduled => {
                return next == CappedIsolationMappingStatus::Cancelled;
            }
            CappedIsolationMappingStatus::InProgress => {
                match next {
                    CappedIsolationMappingStatus::Completed | CappedIsolationMappingStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedIsolationMappingStatus::Completed => {
                return next == CappedIsolationMappingStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedIsolationMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped isolation mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.cost + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedIsolationMapping {
        CappedIsolationMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            visible: self.visible,
            contact_email: self.contact_email.clone(),
            cost: self.cost,
            tenant_id: self.tenant_id.clone(),
            attempts: self.attempts,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped isolation mapping in status planned.
pub fn new_capped_isolation_mapping(code: &str, title: &str) -> CappedIsolationMapping {
    CappedIsolationMapping {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        visible: false,
        contact_email: "".to_string(),
        cost: 0.0,
        tenant_id: "".to_string(),
        attempts: 0,
        host: "".to_string(),
        status: CappedIsolationMappingStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_capped_isolation_mapping("code-855", "title-610");
        assert_eq!(item.status, CappedIsolationMappingStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_capped_isolation_mapping("code-855", "title-610");
        if item.move_to(CappedIsolationMappingStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedIsolationMappingStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_capped_isolation_mapping("code-855", "title-610");
        item.status = CappedIsolationMappingStatus::Cancelled;
        assert!(
            item.move_to(CappedIsolationMappingStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
