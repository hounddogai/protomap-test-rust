use super::*;

use acme_kit::text;

/// Where a capped suspension override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSuspensionOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedSuspensionOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedSuspensionOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped suspension override as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionOverride {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub audience: String,
    pub rate: f64,
    pub summary: String,
    pub status: CappedSuspensionOverrideStatus,
}

impl CappedSuspensionOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSuspensionOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSuspensionOverrideStatus) -> bool {
        match self.status {
            CappedSuspensionOverrideStatus::Proposed => {
                return next == CappedSuspensionOverrideStatus::Approved
                    || next == CappedSuspensionOverrideStatus::Rejected;
            }
            CappedSuspensionOverrideStatus::Approved => {
                return next == CappedSuspensionOverrideStatus::Rejected;
            }
            CappedSuspensionOverrideStatus::Rejected => {
                match next {
                    CappedSuspensionOverrideStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: CappedSuspensionOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped suspension override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped suspension override", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.score * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSuspensionOverride {
        CappedSuspensionOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            audience: self.audience.clone(),
            rate: self.rate,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped suspension override in status proposed.
pub fn new_capped_suspension_override(code: &str, name: &str) -> CappedSuspensionOverride {
    CappedSuspensionOverride {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        audience: "".to_string(),
        rate: 0.0,
        summary: "".to_string(),
        status: CappedSuspensionOverrideStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_suspension_override("code-548", "name-545");
        assert_eq!(item.status, CappedSuspensionOverrideStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_capped_suspension_override("code-548", "name-545");
        if item.move_to(CappedSuspensionOverrideStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedSuspensionOverrideStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_suspension_override("code-548", "name-545");
        item.status = CappedSuspensionOverrideStatus::Withdrawn;
        assert!(
            item.move_to(CappedSuspensionOverrideStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
