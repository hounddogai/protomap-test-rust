use super::*;

use acme_kit::text;

/// Where a fallback identity entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackIdentityEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackIdentityEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackIdentityEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback identity entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityEntry {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub principal: String,
    pub updated_at: i64,
    pub summary: String,
    pub ratio: f64,
    pub segment: String,
    pub grace_days: i64,
    pub amount: f64,
    pub burst: i64,
    pub status: FallbackIdentityEntryStatus,
}

impl FallbackIdentityEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackIdentityEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackIdentityEntryStatus) -> bool {
        match self.status {
            FallbackIdentityEntryStatus::Proposed => {
                match next {
                    FallbackIdentityEntryStatus::Rejected | FallbackIdentityEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackIdentityEntryStatus::Approved => {
                return next == FallbackIdentityEntryStatus::Rejected || next == FallbackIdentityEntryStatus::Withdrawn;
            }
            FallbackIdentityEntryStatus::Rejected => {
                match next {
                    FallbackIdentityEntryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FallbackIdentityEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback identity entry {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackIdentityEntry {
        FallbackIdentityEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            principal: self.principal.clone(),
            updated_at: self.updated_at,
            summary: self.summary.clone(),
            ratio: self.ratio,
            segment: self.segment.clone(),
            grace_days: self.grace_days,
            amount: self.amount,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a fallback identity entry in status proposed.
pub fn new_fallback_identity_entry(code: &str, name: &str) -> FallbackIdentityEntry {
    FallbackIdentityEntry {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        principal: "".to_string(),
        updated_at: 0,
        summary: "".to_string(),
        ratio: 0.0,
        segment: "".to_string(),
        grace_days: 0,
        amount: 0.0,
        burst: 0,
        status: FallbackIdentityEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_fallback_identity_entry("code-68", "name-529");
        assert_eq!(item.status, FallbackIdentityEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_fallback_identity_entry("code-68", "name-529");
        if item.move_to(FallbackIdentityEntryStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackIdentityEntryStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_fallback_identity_entry("code-68", "name-529");
        item.status = FallbackIdentityEntryStatus::Withdrawn;
        assert!(
            item.move_to(FallbackIdentityEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
