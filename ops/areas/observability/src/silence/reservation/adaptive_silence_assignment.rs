use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a adaptive silence assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSilenceAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveSilenceAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveSilenceAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive silence assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceAssignment {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub updated_at: i64,
    pub unit: String,
    pub attributes: BTreeMap<String, String>,
    pub cipher: String,
    pub category: String,
    pub capacity: i64,
    pub status: AdaptiveSilenceAssignmentStatus,
}

impl AdaptiveSilenceAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSilenceAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSilenceAssignmentStatus) -> bool {
        match self.status {
            AdaptiveSilenceAssignmentStatus::Proposed => {
                match next {
                    AdaptiveSilenceAssignmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSilenceAssignmentStatus::Approved => {
                return next == AdaptiveSilenceAssignmentStatus::Withdrawn;
            }
            AdaptiveSilenceAssignmentStatus::Rejected => {
                return next == AdaptiveSilenceAssignmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveSilenceAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive silence assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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
        ((self.updated_at as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveSilenceAssignment {
        AdaptiveSilenceAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            updated_at: self.updated_at,
            unit: self.unit.clone(),
            attributes: self.attributes.clone(),
            cipher: self.cipher.clone(),
            category: self.category.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a adaptive silence assignment in status proposed.
pub fn new_adaptive_silence_assignment(code: &str, title: &str) -> AdaptiveSilenceAssignment {
    AdaptiveSilenceAssignment {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        updated_at: 0,
        unit: "".to_string(),
        attributes: BTreeMap::new(),
        cipher: "".to_string(),
        category: "".to_string(),
        capacity: 0,
        status: AdaptiveSilenceAssignmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_silence_assignment("code-164", "title-380");
        assert_eq!(item.status, AdaptiveSilenceAssignmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_adaptive_silence_assignment("code-164", "title-380");
        if item.move_to(AdaptiveSilenceAssignmentStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveSilenceAssignmentStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_silence_assignment("code-164", "title-380");
        item.status = AdaptiveSilenceAssignmentStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveSilenceAssignmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
