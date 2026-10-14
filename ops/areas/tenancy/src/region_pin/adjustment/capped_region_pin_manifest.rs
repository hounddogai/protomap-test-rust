use super::*;

use acme_kit::text;

/// Where a capped region pin manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRegionPinManifestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedRegionPinManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedRegionPinManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped region pin manifest as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinManifest {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub region: String,
    pub attempts: i64,
    pub principal: String,
    pub minimum: f64,
    pub status: CappedRegionPinManifestStatus,
}

impl CappedRegionPinManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRegionPinManifestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRegionPinManifestStatus) -> bool {
        match self.status {
            CappedRegionPinManifestStatus::Proposed => {
                return next == CappedRegionPinManifestStatus::Withdrawn;
            }
            CappedRegionPinManifestStatus::Approved => {
                return next == CappedRegionPinManifestStatus::Rejected;
            }
            CappedRegionPinManifestStatus::Rejected => {
                return next == CappedRegionPinManifestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedRegionPinManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped region pin manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.attempts as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedRegionPinManifest {
        CappedRegionPinManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            region: self.region.clone(),
            attempts: self.attempts,
            principal: self.principal.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a capped region pin manifest in status proposed.
pub fn new_capped_region_pin_manifest(code: &str, name: &str) -> CappedRegionPinManifest {
    CappedRegionPinManifest {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        region: "".to_string(),
        attempts: 0,
        principal: "".to_string(),
        minimum: 0.0,
        status: CappedRegionPinManifestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_region_pin_manifest("code-805", "name-503");
        assert_eq!(item.status, CappedRegionPinManifestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_capped_region_pin_manifest("code-805", "name-503");
        if item.move_to(CappedRegionPinManifestStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedRegionPinManifestStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_region_pin_manifest("code-805", "name-503");
        item.status = CappedRegionPinManifestStatus::Withdrawn;
        assert!(
            item.move_to(CappedRegionPinManifestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
