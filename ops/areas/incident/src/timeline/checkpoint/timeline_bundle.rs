use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a timeline bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineBundleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TimelineBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TimelineBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One timeline bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBundle {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub maximum: f64,
    pub ends_at: i64,
    pub phone: String,
    pub checksum: String,
    pub cluster: String,
    pub attributes: BTreeMap<String, String>,
    pub amount: f64,
    pub attempts: i64,
    pub status: TimelineBundleStatus,
}

impl TimelineBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineBundleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineBundleStatus) -> bool {
        match self.status {
            TimelineBundleStatus::Created => {
                match next {
                    TimelineBundleStatus::Validated | TimelineBundleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineBundleStatus::Validated => {
                return next == TimelineBundleStatus::Published || next == TimelineBundleStatus::Retired;
            }
            TimelineBundleStatus::Published => {
                match next {
                    TimelineBundleStatus::Retired => {
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

    pub fn move_to(&mut self, next: TimelineBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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

    pub fn projected_value(&self) -> f64 {
        ((self.attempts as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineBundle {
        TimelineBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            maximum: self.maximum,
            ends_at: self.ends_at,
            phone: self.phone.clone(),
            checksum: self.checksum.clone(),
            cluster: self.cluster.clone(),
            attributes: self.attributes.clone(),
            amount: self.amount,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a timeline bundle in status created.
pub fn new_timeline_bundle(code: &str, title: &str) -> TimelineBundle {
    TimelineBundle {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        maximum: 0.0,
        ends_at: 0,
        phone: "".to_string(),
        checksum: "".to_string(),
        cluster: "".to_string(),
        attributes: BTreeMap::new(),
        amount: 0.0,
        attempts: 0,
        status: TimelineBundleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_timeline_bundle("code-84", "title-372");
        assert_eq!(item.status, TimelineBundleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_timeline_bundle("code-84", "title-372");
        if item.move_to(TimelineBundleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineBundleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_timeline_bundle("code-84", "title-372");
        item.status = TimelineBundleStatus::Retired;
        assert!(
            item.move_to(TimelineBundleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
