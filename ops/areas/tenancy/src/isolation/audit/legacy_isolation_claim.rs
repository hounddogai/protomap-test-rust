use super::*;

use acme_kit::text;

/// Where a legacy isolation claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyIsolationClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyIsolationClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyIsolationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy isolation claim as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationClaim {
    pub code: String,
    pub label: String,
    pub category: String,
    pub region: String,
    pub revision: i64,
    pub created_at: i64,
    pub summary: String,
    pub limit: i64,
    pub phone: String,
    pub title: String,
    pub status: LegacyIsolationClaimStatus,
}

impl LegacyIsolationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyIsolationClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyIsolationClaimStatus) -> bool {
        match self.status {
            LegacyIsolationClaimStatus::Proposed => {
                return next == LegacyIsolationClaimStatus::Rejected || next == LegacyIsolationClaimStatus::Withdrawn;
            }
            LegacyIsolationClaimStatus::Approved => {
                return next == LegacyIsolationClaimStatus::Rejected;
            }
            LegacyIsolationClaimStatus::Rejected => {
                match next {
                    LegacyIsolationClaimStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LegacyIsolationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy isolation claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy isolation claim", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.revision as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyIsolationClaim {
        LegacyIsolationClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            region: self.region.clone(),
            revision: self.revision,
            created_at: self.created_at,
            summary: self.summary.clone(),
            limit: self.limit,
            phone: self.phone.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a legacy isolation claim in status proposed.
pub fn new_legacy_isolation_claim(code: &str, label: &str) -> LegacyIsolationClaim {
    LegacyIsolationClaim {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        region: "".to_string(),
        revision: 0,
        created_at: 0,
        summary: "".to_string(),
        limit: 0,
        phone: "".to_string(),
        title: "".to_string(),
        status: LegacyIsolationClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_legacy_isolation_claim("code-352", "label-552");
        assert_eq!(item.status, LegacyIsolationClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_legacy_isolation_claim("code-352", "label-552");
        if item.move_to(LegacyIsolationClaimStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyIsolationClaimStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_legacy_isolation_claim("code-352", "label-552");
        item.status = LegacyIsolationClaimStatus::Withdrawn;
        assert!(
            item.move_to(LegacyIsolationClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
