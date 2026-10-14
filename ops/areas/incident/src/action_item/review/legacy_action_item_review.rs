use super::*;

use acme_kit::text;

/// Where a legacy action item review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyActionItemReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyActionItemReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyActionItemReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy action item review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemReview {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub namespace: String,
    pub quantity: i64,
    pub rotation_days: i64,
    pub maximum: f64,
    pub weight: f64,
    pub status: LegacyActionItemReviewStatus,
}

impl LegacyActionItemReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyActionItemReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyActionItemReviewStatus) -> bool {
        match self.status {
            LegacyActionItemReviewStatus::Requested => {
                match next {
                    LegacyActionItemReviewStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyActionItemReviewStatus::Granted => {
                match next {
                    LegacyActionItemReviewStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyActionItemReviewStatus::Revoked => {
                match next {
                    LegacyActionItemReviewStatus::Expired => {
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

    pub fn move_to(&mut self, next: LegacyActionItemReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy action item review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.rotation_days as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyActionItemReview {
        LegacyActionItemReview {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            namespace: self.namespace.clone(),
            quantity: self.quantity,
            rotation_days: self.rotation_days,
            maximum: self.maximum,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a legacy action item review in status requested.
pub fn new_legacy_action_item_review(code: &str, label: &str) -> LegacyActionItemReview {
    LegacyActionItemReview {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        namespace: "".to_string(),
        quantity: 0,
        rotation_days: 0,
        maximum: 0.0,
        weight: 0.0,
        status: LegacyActionItemReviewStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_legacy_action_item_review("code-554", "label-748");
        assert_eq!(item.status, LegacyActionItemReviewStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_legacy_action_item_review("code-554", "label-748");
        if item.move_to(LegacyActionItemReviewStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyActionItemReviewStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_legacy_action_item_review("code-554", "label-748");
        item.status = LegacyActionItemReviewStatus::Expired;
        assert!(
            item.move_to(LegacyActionItemReviewStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
