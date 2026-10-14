use super::*;

use acme_kit::text;

/// Where a grant schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GrantScheduleStatus {
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

impl std::fmt::Display for GrantScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a grant schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantSchedule {
    pub code: String,
    pub name: String,
    pub description: String,
    pub issuer: String,
    pub port: i64,
    pub tags: Vec<String>,
    pub key_id: String,
    pub margin: f64,
    pub category: String,
    pub label: String,
    pub status: GrantScheduleStatus,
}

impl GrantSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantScheduleStatus) -> bool {
        match self.status {
            GrantScheduleStatus::Open => {
                match next {
                    GrantScheduleStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantScheduleStatus::InReview => {
                return next == GrantScheduleStatus::Reopened || next == GrantScheduleStatus::Archived;
            }
            GrantScheduleStatus::Resolved => {
                match next {
                    GrantScheduleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantScheduleStatus::Reopened => {
                match next {
                    GrantScheduleStatus::Archived => {
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

    pub fn move_to(&mut self, next: GrantScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "grant schedule", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.port as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantSchedule {
        GrantSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            issuer: self.issuer.clone(),
            port: self.port,
            tags: self.tags.clone(),
            key_id: self.key_id.clone(),
            margin: self.margin,
            category: self.category.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a grant schedule in status open.
pub fn new_grant_schedule(code: &str, name: &str) -> GrantSchedule {
    GrantSchedule {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        issuer: "".to_string(),
        port: 0,
        tags: Vec::new(),
        key_id: "".to_string(),
        margin: 0.0,
        category: "".to_string(),
        label: "".to_string(),
        status: GrantScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_grant_schedule("code-329", "name-648");
        assert_eq!(item.status, GrantScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_grant_schedule("code-329", "name-648");
        if item.move_to(GrantScheduleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantScheduleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_grant_schedule("code-329", "name-648");
        item.status = GrantScheduleStatus::Archived;
        assert!(
            item.move_to(GrantScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
