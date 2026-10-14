use super::*;

use acme_kit::text;

/// Where a suspension record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SuspensionRecordStatus {
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

impl std::fmt::Display for SuspensionRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRecord {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub cluster: String,
    pub tags: Vec<String>,
    pub country: String,
    pub batch_size: i64,
    pub limit: i64,
    pub status: SuspensionRecordStatus,
}

impl SuspensionRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionRecordStatus) -> bool {
        match self.status {
            SuspensionRecordStatus::Open => {
                return next == SuspensionRecordStatus::Resolved || next == SuspensionRecordStatus::Reopened;
            }
            SuspensionRecordStatus::InReview => {
                match next {
                    SuspensionRecordStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionRecordStatus::Resolved => {
                match next {
                    SuspensionRecordStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionRecordStatus::Reopened => {
                return next == SuspensionRecordStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.limit as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionRecord {
        SuspensionRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            cluster: self.cluster.clone(),
            tags: self.tags.clone(),
            country: self.country.clone(),
            batch_size: self.batch_size,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a suspension record in status open.
pub fn new_suspension_record(code: &str, name: &str) -> SuspensionRecord {
    SuspensionRecord {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        cluster: "".to_string(),
        tags: Vec::new(),
        country: "".to_string(),
        batch_size: 0,
        limit: 0,
        status: SuspensionRecordStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_suspension_record("code-615", "name-818");
        assert_eq!(item.status, SuspensionRecordStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_suspension_record("code-615", "name-818");
        if item.move_to(SuspensionRecordStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionRecordStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_suspension_record("code-615", "name-818");
        item.status = SuspensionRecordStatus::Archived;
        assert!(
            item.move_to(SuspensionRecordStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
