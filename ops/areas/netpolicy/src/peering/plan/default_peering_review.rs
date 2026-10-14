use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default peering review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultPeeringReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultPeeringReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultPeeringReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default peering review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringReview {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub attributes: BTreeMap<String, String>,
    pub cost: f64,
    pub ttl_seconds: i64,
    pub revision: i64,
    pub country: String,
    pub issuer: String,
    pub visible: bool,
    pub status: DefaultPeeringReviewStatus,
}

impl DefaultPeeringReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultPeeringReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultPeeringReviewStatus) -> bool {
        match self.status {
            DefaultPeeringReviewStatus::Proposed => {
                return next == DefaultPeeringReviewStatus::Approved || next == DefaultPeeringReviewStatus::Rejected;
            }
            DefaultPeeringReviewStatus::Approved => {
                match next {
                    DefaultPeeringReviewStatus::Rejected | DefaultPeeringReviewStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultPeeringReviewStatus::Rejected => {
                return next == DefaultPeeringReviewStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultPeeringReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default peering review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn weighted_value(&self) -> f64 {
        ((self.cost + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultPeeringReview {
        DefaultPeeringReview {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            attributes: self.attributes.clone(),
            cost: self.cost,
            ttl_seconds: self.ttl_seconds,
            revision: self.revision,
            country: self.country.clone(),
            issuer: self.issuer.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a default peering review in status proposed.
pub fn new_default_peering_review(code: &str, label: &str) -> DefaultPeeringReview {
    DefaultPeeringReview {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        attributes: BTreeMap::new(),
        cost: 0.0,
        ttl_seconds: 0,
        revision: 0,
        country: "".to_string(),
        issuer: "".to_string(),
        visible: false,
        status: DefaultPeeringReviewStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_peering_review("code-350", "label-787");
        assert_eq!(item.status, DefaultPeeringReviewStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_default_peering_review("code-350", "label-787");
        if item.move_to(DefaultPeeringReviewStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultPeeringReviewStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_peering_review("code-350", "label-787");
        item.status = DefaultPeeringReviewStatus::Withdrawn;
        assert!(
            item.move_to(DefaultPeeringReviewStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
