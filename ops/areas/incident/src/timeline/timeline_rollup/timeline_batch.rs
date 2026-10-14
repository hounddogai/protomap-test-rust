use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a timeline batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineBatchStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TimelineBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TimelineBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline batch as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBatch {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub attributes: BTreeMap<String, String>,
    pub country: String,
    pub channel: String,
    pub minimum: f64,
    pub ceiling: f64,
    pub name: String,
    pub margin: f64,
    pub status: TimelineBatchStatus,
}

impl TimelineBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineBatchStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineBatchStatus) -> bool {
        match self.status {
            TimelineBatchStatus::Proposed => {
                return next == TimelineBatchStatus::Rejected || next == TimelineBatchStatus::Withdrawn;
            }
            TimelineBatchStatus::Approved => {
                return next == TimelineBatchStatus::Rejected;
            }
            TimelineBatchStatus::Rejected => {
                match next {
                    TimelineBatchStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TimelineBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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

    pub fn effective_value(&self) -> f64 {
        ((self.minimum - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineBatch {
        TimelineBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            attributes: self.attributes.clone(),
            country: self.country.clone(),
            channel: self.channel.clone(),
            minimum: self.minimum,
            ceiling: self.ceiling,
            name: self.name.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a timeline batch in status proposed.
pub fn new_timeline_batch(code: &str, title: &str) -> TimelineBatch {
    TimelineBatch {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        attributes: BTreeMap::new(),
        country: "".to_string(),
        channel: "".to_string(),
        minimum: 0.0,
        ceiling: 0.0,
        name: "".to_string(),
        margin: 0.0,
        status: TimelineBatchStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_timeline_batch("code-830", "title-696");
        assert_eq!(item.status, TimelineBatchStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_timeline_batch("code-830", "title-696");
        if item.move_to(TimelineBatchStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineBatchStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_timeline_batch("code-830", "title-696");
        item.status = TimelineBatchStatus::Withdrawn;
        assert!(
            item.move_to(TimelineBatchStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
