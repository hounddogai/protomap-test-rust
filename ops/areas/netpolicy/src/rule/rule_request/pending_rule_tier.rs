use super::*;

use acme_kit::text;

/// Where a pending rule tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRuleTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingRuleTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PendingRuleTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending rule tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleTier {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub unit: String,
    pub summary: String,
    pub capacity: i64,
    pub tags: Vec<String>,
    pub currency: String,
    pub rotation_days: i64,
    pub budget: f64,
    pub status: PendingRuleTierStatus,
}

impl PendingRuleTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRuleTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRuleTierStatus) -> bool {
        match self.status {
            PendingRuleTierStatus::Draft => {
                return next == PendingRuleTierStatus::Pending || next == PendingRuleTierStatus::Active;
            }
            PendingRuleTierStatus::Pending => {
                match next {
                    PendingRuleTierStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRuleTierStatus::Active => {
                match next {
                    PendingRuleTierStatus::Suspended | PendingRuleTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRuleTierStatus::Suspended => {
                match next {
                    PendingRuleTierStatus::Closed => {
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

    pub fn move_to(&mut self, next: PendingRuleTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending rule tier {} cannot move from {} to {}",
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

    pub fn adjusted_load(&self) -> f64 {
        (self.budget * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRuleTier {
        PendingRuleTier {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            unit: self.unit.clone(),
            summary: self.summary.clone(),
            capacity: self.capacity,
            tags: self.tags.clone(),
            currency: self.currency.clone(),
            rotation_days: self.rotation_days,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a pending rule tier in status draft.
pub fn new_pending_rule_tier(code: &str, label: &str) -> PendingRuleTier {
    PendingRuleTier {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        unit: "".to_string(),
        summary: "".to_string(),
        capacity: 0,
        tags: Vec::new(),
        currency: "".to_string(),
        rotation_days: 0,
        budget: 0.0,
        status: PendingRuleTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_rule_tier("code-359", "label-310");
        assert_eq!(item.status, PendingRuleTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_pending_rule_tier("code-359", "label-310");
        if item.move_to(PendingRuleTierStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRuleTierStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_rule_tier("code-359", "label-310");
        item.status = PendingRuleTierStatus::Closed;
        assert!(
            item.move_to(PendingRuleTierStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
