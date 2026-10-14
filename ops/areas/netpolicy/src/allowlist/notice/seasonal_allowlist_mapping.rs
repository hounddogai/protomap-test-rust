use super::*;

use acme_kit::text;

/// Where a seasonal allowlist mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalAllowlistMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalAllowlistMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalAllowlistMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal allowlist mapping as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistMapping {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub principal: String,
    pub pinned: bool,
    pub discount: f64,
    pub status: SeasonalAllowlistMappingStatus,
}

impl SeasonalAllowlistMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalAllowlistMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalAllowlistMappingStatus) -> bool {
        match self.status {
            SeasonalAllowlistMappingStatus::Proposed => {
                return next == SeasonalAllowlistMappingStatus::Rejected;
            }
            SeasonalAllowlistMappingStatus::Approved => {
                match next {
                    SeasonalAllowlistMappingStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalAllowlistMappingStatus::Rejected => {
                return next == SeasonalAllowlistMappingStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalAllowlistMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal allowlist mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.floor_value + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalAllowlistMapping {
        SeasonalAllowlistMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            principal: self.principal.clone(),
            pinned: self.pinned,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal allowlist mapping in status proposed.
pub fn new_seasonal_allowlist_mapping(code: &str, label: &str) -> SeasonalAllowlistMapping {
    SeasonalAllowlistMapping {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        principal: "".to_string(),
        pinned: false,
        discount: 0.0,
        status: SeasonalAllowlistMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_allowlist_mapping("code-964", "label-176");
        assert_eq!(item.status, SeasonalAllowlistMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_seasonal_allowlist_mapping("code-964", "label-176");
        if item.move_to(SeasonalAllowlistMappingStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalAllowlistMappingStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_allowlist_mapping("code-964", "label-176");
        item.status = SeasonalAllowlistMappingStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalAllowlistMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
