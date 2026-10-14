use super::*;

use acme_kit::text;

/// Where a strict region pin tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRegionPinTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictRegionPinTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for StrictRegionPinTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict region pin tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRegionPinTier {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub fingerprint: String,
    pub name: String,
    pub weight: f64,
    pub reviewer: String,
    pub approved: bool,
    pub ttl_seconds: i64,
    pub version: i64,
    pub floor_value: f64,
    pub status: StrictRegionPinTierStatus,
}

impl StrictRegionPinTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRegionPinTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRegionPinTierStatus) -> bool {
        match self.status {
            StrictRegionPinTierStatus::Open => {
                match next {
                    StrictRegionPinTierStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRegionPinTierStatus::InReview => {
                return next == StrictRegionPinTierStatus::Resolved || next == StrictRegionPinTierStatus::Archived;
            }
            StrictRegionPinTierStatus::Resolved => {
                return next == StrictRegionPinTierStatus::Archived;
            }
            StrictRegionPinTierStatus::Reopened => {
                return next == StrictRegionPinTierStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictRegionPinTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict region pin tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict region pin tier", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        (self.floor_value * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictRegionPinTier {
        StrictRegionPinTier {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            fingerprint: self.fingerprint.clone(),
            name: self.name.clone(),
            weight: self.weight,
            reviewer: self.reviewer.clone(),
            approved: self.approved,
            ttl_seconds: self.ttl_seconds,
            version: self.version,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict region pin tier in status open.
pub fn new_strict_region_pin_tier(code: &str, title: &str) -> StrictRegionPinTier {
    StrictRegionPinTier {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        fingerprint: "".to_string(),
        name: "".to_string(),
        weight: 0.0,
        reviewer: "".to_string(),
        approved: false,
        ttl_seconds: 0,
        version: 0,
        floor_value: 0.0,
        status: StrictRegionPinTierStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_region_pin_tier("code-504", "title-901");
        assert_eq!(item.status, StrictRegionPinTierStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_strict_region_pin_tier("code-504", "title-901");
        if item.move_to(StrictRegionPinTierStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRegionPinTierStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_region_pin_tier("code-504", "title-901");
        item.status = StrictRegionPinTierStatus::Archived;
        assert!(
            item.move_to(StrictRegionPinTierStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
