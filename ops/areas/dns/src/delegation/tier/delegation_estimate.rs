use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a delegation estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DelegationEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DelegationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEstimate {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub tags: Vec<String>,
    pub attributes: BTreeMap<String, String>,
    pub score: f64,
    pub archived: bool,
    pub quantity: i64,
    pub locale: String,
    pub region: String,
    pub country: String,
    pub status: DelegationEstimateStatus,
}

impl DelegationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationEstimateStatus) -> bool {
        match self.status {
            DelegationEstimateStatus::Created => {
                return next == DelegationEstimateStatus::Published;
            }
            DelegationEstimateStatus::Validated => {
                match next {
                    DelegationEstimateStatus::Published | DelegationEstimateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationEstimateStatus::Published => {
                match next {
                    DelegationEstimateStatus::Retired => {
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

    pub fn move_to(&mut self, next: DelegationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn adjusted_value(&self) -> f64 {
        ((self.budget + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationEstimate {
        DelegationEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            tags: self.tags.clone(),
            attributes: self.attributes.clone(),
            score: self.score,
            archived: self.archived,
            quantity: self.quantity,
            locale: self.locale.clone(),
            region: self.region.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a delegation estimate in status created.
pub fn new_delegation_estimate(code: &str, label: &str) -> DelegationEstimate {
    DelegationEstimate {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        tags: Vec::new(),
        attributes: BTreeMap::new(),
        score: 0.0,
        archived: false,
        quantity: 0,
        locale: "".to_string(),
        region: "".to_string(),
        country: "".to_string(),
        status: DelegationEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_delegation_estimate("code-248", "label-884");
        assert_eq!(item.status, DelegationEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_delegation_estimate("code-248", "label-884");
        if item.move_to(DelegationEstimateStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationEstimateStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_delegation_estimate("code-248", "label-884");
        item.status = DelegationEstimateStatus::Retired;
        assert!(
            item.move_to(DelegationEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
