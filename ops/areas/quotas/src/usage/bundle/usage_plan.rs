use super::*;

use acme_kit::text;

/// Where a usage plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsagePlanStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl UsagePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for UsagePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a usage plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct UsagePlan {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub audience: String,
    pub ceiling: f64,
    pub cipher: String,
    pub key_id: String,
    pub margin: f64,
    pub status: UsagePlanStatus,
}

impl UsagePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsagePlanStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsagePlanStatus) -> bool {
        match self.status {
            UsagePlanStatus::Proposed => {
                match next {
                    UsagePlanStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsagePlanStatus::Approved => {
                match next {
                    UsagePlanStatus::Rejected | UsagePlanStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsagePlanStatus::Rejected => {
                return next == UsagePlanStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsagePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.margin - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsagePlan {
        UsagePlan {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            audience: self.audience.clone(),
            ceiling: self.ceiling,
            cipher: self.cipher.clone(),
            key_id: self.key_id.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a usage plan in status proposed.
pub fn new_usage_plan(code: &str, title: &str) -> UsagePlan {
    UsagePlan {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        audience: "".to_string(),
        ceiling: 0.0,
        cipher: "".to_string(),
        key_id: "".to_string(),
        margin: 0.0,
        status: UsagePlanStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_usage_plan("code-411", "title-313");
        assert_eq!(item.status, UsagePlanStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_usage_plan("code-411", "title-313");
        if item.move_to(UsagePlanStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsagePlanStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_usage_plan("code-411", "title-313");
        item.status = UsagePlanStatus::Withdrawn;
        assert!(
            item.move_to(UsagePlanStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
