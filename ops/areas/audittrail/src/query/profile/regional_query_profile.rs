use super::*;

use acme_kit::text;

/// Where a regional query profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalQueryProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalQueryProfileStatus {
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

impl std::fmt::Display for RegionalQueryProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional query profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryProfile {
    pub code: String,
    pub name: String,
    pub category: String,
    pub source: String,
    pub tags: Vec<String>,
    pub port: i64,
    pub notes: String,
    pub floor_value: f64,
    pub ttl_seconds: i64,
    pub amount: f64,
    pub status: RegionalQueryProfileStatus,
}

impl RegionalQueryProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalQueryProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalQueryProfileStatus) -> bool {
        match self.status {
            RegionalQueryProfileStatus::Open => {
                match next {
                    RegionalQueryProfileStatus::Resolved | RegionalQueryProfileStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalQueryProfileStatus::InReview => {
                return next == RegionalQueryProfileStatus::Reopened || next == RegionalQueryProfileStatus::Archived;
            }
            RegionalQueryProfileStatus::Resolved => {
                match next {
                    RegionalQueryProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalQueryProfileStatus::Reopened => {
                return next == RegionalQueryProfileStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalQueryProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional query profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.floor_value - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalQueryProfile {
        RegionalQueryProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            source: self.source.clone(),
            tags: self.tags.clone(),
            port: self.port,
            notes: self.notes.clone(),
            floor_value: self.floor_value,
            ttl_seconds: self.ttl_seconds,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a regional query profile in status open.
pub fn new_regional_query_profile(code: &str, name: &str) -> RegionalQueryProfile {
    RegionalQueryProfile {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        source: "".to_string(),
        tags: Vec::new(),
        port: 0,
        notes: "".to_string(),
        floor_value: 0.0,
        ttl_seconds: 0,
        amount: 0.0,
        status: RegionalQueryProfileStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_query_profile("code-36", "name-276");
        assert_eq!(item.status, RegionalQueryProfileStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_regional_query_profile("code-36", "name-276");
        if item.move_to(RegionalQueryProfileStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalQueryProfileStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_query_profile("code-36", "name-276");
        item.status = RegionalQueryProfileStatus::Archived;
        assert!(
            item.move_to(RegionalQueryProfileStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
