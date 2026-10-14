use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a regional binding review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalBindingReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalBindingReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalBindingReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional binding review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBindingReview {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub unit: String,
    pub reviewer: String,
    pub summary: String,
    pub score: f64,
    pub external_ref: String,
    pub minimum: f64,
    pub active: bool,
    pub description: String,
    pub status: RegionalBindingReviewStatus,
}

impl RegionalBindingReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalBindingReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalBindingReviewStatus) -> bool {
        match self.status {
            RegionalBindingReviewStatus::Created => {
                match next {
                    RegionalBindingReviewStatus::Published | RegionalBindingReviewStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalBindingReviewStatus::Validated => {
                return next == RegionalBindingReviewStatus::Published || next == RegionalBindingReviewStatus::Retired;
            }
            RegionalBindingReviewStatus::Published => {
                return next == RegionalBindingReviewStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalBindingReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional binding review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional binding review", self.code)
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

    pub fn effective_load(&self) -> f64 {
        ((self.score + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalBindingReview {
        RegionalBindingReview {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            unit: self.unit.clone(),
            reviewer: self.reviewer.clone(),
            summary: self.summary.clone(),
            score: self.score,
            external_ref: self.external_ref.clone(),
            minimum: self.minimum,
            active: self.active,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a regional binding review in status created.
pub fn new_regional_binding_review(code: &str, label: &str) -> RegionalBindingReview {
    RegionalBindingReview {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        unit: "".to_string(),
        reviewer: "".to_string(),
        summary: "".to_string(),
        score: 0.0,
        external_ref: "".to_string(),
        minimum: 0.0,
        active: false,
        description: "".to_string(),
        status: RegionalBindingReviewStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_binding_review("code-141", "label-36");
        assert_eq!(item.status, RegionalBindingReviewStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_regional_binding_review("code-141", "label-36");
        if item.move_to(RegionalBindingReviewStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalBindingReviewStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_binding_review("code-141", "label-36");
        item.status = RegionalBindingReviewStatus::Retired;
        assert!(
            item.move_to(RegionalBindingReviewStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
