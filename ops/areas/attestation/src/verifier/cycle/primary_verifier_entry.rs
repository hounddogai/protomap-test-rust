use super::*;

use acme_kit::text;

/// Where a primary verifier entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryVerifierEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryVerifierEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryVerifierEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary verifier entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierEntry {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub checksum: String,
    pub tags: Vec<String>,
    pub threshold: f64,
    pub region: String,
    pub tenant_id: String,
    pub budget: f64,
    pub status: PrimaryVerifierEntryStatus,
}

impl PrimaryVerifierEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryVerifierEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryVerifierEntryStatus) -> bool {
        match self.status {
            PrimaryVerifierEntryStatus::Proposed => {
                return next == PrimaryVerifierEntryStatus::Approved || next == PrimaryVerifierEntryStatus::Rejected;
            }
            PrimaryVerifierEntryStatus::Approved => {
                match next {
                    PrimaryVerifierEntryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryVerifierEntryStatus::Rejected => {
                return next == PrimaryVerifierEntryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryVerifierEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary verifier entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary verifier entry", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_load(&self) -> f64 {
        ((self.budget + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryVerifierEntry {
        PrimaryVerifierEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            checksum: self.checksum.clone(),
            tags: self.tags.clone(),
            threshold: self.threshold,
            region: self.region.clone(),
            tenant_id: self.tenant_id.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary verifier entry in status proposed.
pub fn new_primary_verifier_entry(code: &str, name: &str) -> PrimaryVerifierEntry {
    PrimaryVerifierEntry {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        checksum: "".to_string(),
        tags: Vec::new(),
        threshold: 0.0,
        region: "".to_string(),
        tenant_id: "".to_string(),
        budget: 0.0,
        status: PrimaryVerifierEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_verifier_entry("code-69", "name-828");
        assert_eq!(item.status, PrimaryVerifierEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_primary_verifier_entry("code-69", "name-828");
        if item.move_to(PrimaryVerifierEntryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryVerifierEntryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_verifier_entry("code-69", "name-828");
        item.status = PrimaryVerifierEntryStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryVerifierEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
