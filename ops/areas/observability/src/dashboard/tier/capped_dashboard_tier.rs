use super::*;

use acme_kit::text;

/// Where a capped dashboard tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedDashboardTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedDashboardTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedDashboardTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped dashboard tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTier {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub quantity: i64,
    pub issuer: String,
    pub maximum: f64,
    pub weight: f64,
    pub reviewer: String,
    pub category: String,
    pub tags: Vec<String>,
    pub archived: bool,
    pub status: CappedDashboardTierStatus,
}

impl CappedDashboardTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedDashboardTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedDashboardTierStatus) -> bool {
        match self.status {
            CappedDashboardTierStatus::Requested => {
                match next {
                    CappedDashboardTierStatus::Granted | CappedDashboardTierStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedDashboardTierStatus::Granted => {
                match next {
                    CappedDashboardTierStatus::Revoked | CappedDashboardTierStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedDashboardTierStatus::Revoked => {
                match next {
                    CappedDashboardTierStatus::Expired => {
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

    pub fn move_to(&mut self, next: CappedDashboardTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped dashboard tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.maximum + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedDashboardTier {
        CappedDashboardTier {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            quantity: self.quantity,
            issuer: self.issuer.clone(),
            maximum: self.maximum,
            weight: self.weight,
            reviewer: self.reviewer.clone(),
            category: self.category.clone(),
            tags: self.tags.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a capped dashboard tier in status requested.
pub fn new_capped_dashboard_tier(code: &str, label: &str) -> CappedDashboardTier {
    CappedDashboardTier {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        quantity: 0,
        issuer: "".to_string(),
        maximum: 0.0,
        weight: 0.0,
        reviewer: "".to_string(),
        category: "".to_string(),
        tags: Vec::new(),
        archived: false,
        status: CappedDashboardTierStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_dashboard_tier("code-940", "label-988");
        assert_eq!(item.status, CappedDashboardTierStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_capped_dashboard_tier("code-940", "label-988");
        if item.move_to(CappedDashboardTierStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedDashboardTierStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_dashboard_tier("code-940", "label-988");
        item.status = CappedDashboardTierStatus::Expired;
        assert!(
            item.move_to(CappedDashboardTierStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
