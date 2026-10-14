use super::*;

use acme_kit::text;

/// Where a default redaction schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRedactionScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultRedactionScheduleStatus {
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

impl std::fmt::Display for DefaultRedactionScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default redaction schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionSchedule {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub source: String,
    pub external_ref: String,
    pub owner_id: String,
    pub cost: f64,
    pub tags: Vec<String>,
    pub score: f64,
    pub discount: f64,
    pub unit: String,
    pub status: DefaultRedactionScheduleStatus,
}

impl DefaultRedactionSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRedactionScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRedactionScheduleStatus) -> bool {
        match self.status {
            DefaultRedactionScheduleStatus::Open => {
                return next == DefaultRedactionScheduleStatus::InReview;
            }
            DefaultRedactionScheduleStatus::InReview => {
                return next == DefaultRedactionScheduleStatus::Archived;
            }
            DefaultRedactionScheduleStatus::Resolved => {
                match next {
                    DefaultRedactionScheduleStatus::Reopened | DefaultRedactionScheduleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRedactionScheduleStatus::Reopened => {
                match next {
                    DefaultRedactionScheduleStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultRedactionScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default redaction schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_volume(&self) -> f64 {
        ((self.score - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRedactionSchedule {
        DefaultRedactionSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            source: self.source.clone(),
            external_ref: self.external_ref.clone(),
            owner_id: self.owner_id.clone(),
            cost: self.cost,
            tags: self.tags.clone(),
            score: self.score,
            discount: self.discount,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a default redaction schedule in status open.
pub fn new_default_redaction_schedule(code: &str, label: &str) -> DefaultRedactionSchedule {
    DefaultRedactionSchedule {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        source: "".to_string(),
        external_ref: "".to_string(),
        owner_id: "".to_string(),
        cost: 0.0,
        tags: Vec::new(),
        score: 0.0,
        discount: 0.0,
        unit: "".to_string(),
        status: DefaultRedactionScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_redaction_schedule("code-132", "label-178");
        assert_eq!(item.status, DefaultRedactionScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_default_redaction_schedule("code-132", "label-178");
        if item.move_to(DefaultRedactionScheduleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRedactionScheduleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_redaction_schedule("code-132", "label-178");
        item.status = DefaultRedactionScheduleStatus::Archived;
        assert!(
            item.move_to(DefaultRedactionScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
