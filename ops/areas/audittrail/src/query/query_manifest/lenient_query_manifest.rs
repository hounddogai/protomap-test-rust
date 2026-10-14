use super::*;

use acme_kit::text;

/// Where a lenient query manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientQueryManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientQueryManifestStatus {
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

impl std::fmt::Display for LenientQueryManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient query manifest as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryManifest {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub priority: i64,
    pub archived: bool,
    pub principal: String,
    pub owner_id: String,
    pub status: LenientQueryManifestStatus,
}

impl LenientQueryManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientQueryManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientQueryManifestStatus) -> bool {
        match self.status {
            LenientQueryManifestStatus::Planned => {
                match next {
                    LenientQueryManifestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientQueryManifestStatus::Scheduled => {
                return next == LenientQueryManifestStatus::Completed || next == LenientQueryManifestStatus::Cancelled;
            }
            LenientQueryManifestStatus::InProgress => {
                match next {
                    LenientQueryManifestStatus::Completed | LenientQueryManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientQueryManifestStatus::Completed => {
                return next == LenientQueryManifestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientQueryManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient query manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> LenientQueryManifest {
        LenientQueryManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            priority: self.priority,
            archived: self.archived,
            principal: self.principal.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a lenient query manifest in status planned.
pub fn new_lenient_query_manifest(code: &str, name: &str) -> LenientQueryManifest {
    LenientQueryManifest {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        priority: 0,
        archived: false,
        principal: "".to_string(),
        owner_id: "".to_string(),
        status: LenientQueryManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_query_manifest("code-344", "name-466");
        assert_eq!(item.status, LenientQueryManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_lenient_query_manifest("code-344", "name-466");
        if item.move_to(LenientQueryManifestStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientQueryManifestStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_query_manifest("code-344", "name-466");
        item.status = LenientQueryManifestStatus::Cancelled;
        assert!(
            item.move_to(LenientQueryManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
