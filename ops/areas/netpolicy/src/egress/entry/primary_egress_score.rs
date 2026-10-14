use super::*;

use acme_kit::text;

/// Where a primary egress score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEgressScoreStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryEgressScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryEgressScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary egress score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressScore {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub pinned: bool,
    pub sequence: i64,
    pub tags: Vec<String>,
    pub floor_value: f64,
    pub grace_days: i64,
    pub key_id: String,
    pub status: PrimaryEgressScoreStatus,
}

impl PrimaryEgressScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEgressScoreStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEgressScoreStatus) -> bool {
        match self.status {
            PrimaryEgressScoreStatus::Proposed => {
                match next {
                    PrimaryEgressScoreStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEgressScoreStatus::Approved => {
                match next {
                    PrimaryEgressScoreStatus::Rejected | PrimaryEgressScoreStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEgressScoreStatus::Rejected => {
                match next {
                    PrimaryEgressScoreStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PrimaryEgressScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary egress score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.grace_days as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryEgressScore {
        PrimaryEgressScore {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            pinned: self.pinned,
            sequence: self.sequence,
            tags: self.tags.clone(),
            floor_value: self.floor_value,
            grace_days: self.grace_days,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary egress score in status proposed.
pub fn new_primary_egress_score(code: &str, title: &str) -> PrimaryEgressScore {
    PrimaryEgressScore {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        pinned: false,
        sequence: 0,
        tags: Vec::new(),
        floor_value: 0.0,
        grace_days: 0,
        key_id: "".to_string(),
        status: PrimaryEgressScoreStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_egress_score("code-542", "title-105");
        assert_eq!(item.status, PrimaryEgressScoreStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_primary_egress_score("code-542", "title-105");
        if item.move_to(PrimaryEgressScoreStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryEgressScoreStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_egress_score("code-542", "title-105");
        item.status = PrimaryEgressScoreStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryEgressScoreStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
