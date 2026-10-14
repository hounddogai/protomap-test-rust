use super::*;

use acme_kit::text;

/// Where a regional retention tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRetentionTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalRetentionTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalRetentionTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional retention tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionTier {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub rotation_days: i64,
    pub ratio: f64,
    pub retries: i64,
    pub status: RegionalRetentionTierStatus,
}

impl RegionalRetentionTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRetentionTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRetentionTierStatus) -> bool {
        match self.status {
            RegionalRetentionTierStatus::Proposed => {
                return next == RegionalRetentionTierStatus::Approved || next == RegionalRetentionTierStatus::Rejected;
            }
            RegionalRetentionTierStatus::Approved => {
                match next {
                    RegionalRetentionTierStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRetentionTierStatus::Rejected => {
                match next {
                    RegionalRetentionTierStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionalRetentionTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional retention tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rotation_days as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRetentionTier {
        RegionalRetentionTier {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            rotation_days: self.rotation_days,
            ratio: self.ratio,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional retention tier in status proposed.
pub fn new_regional_retention_tier(code: &str, title: &str) -> RegionalRetentionTier {
    RegionalRetentionTier {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        rotation_days: 0,
        ratio: 0.0,
        retries: 0,
        status: RegionalRetentionTierStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_retention_tier("code-581", "title-853");
        assert_eq!(item.status, RegionalRetentionTierStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_regional_retention_tier("code-581", "title-853");
        if item.move_to(RegionalRetentionTierStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRetentionTierStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_retention_tier("code-581", "title-853");
        item.status = RegionalRetentionTierStatus::Withdrawn;
        assert!(
            item.move_to(RegionalRetentionTierStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
