use super::*;

use acme_kit::text;

/// Where a isolation bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationBundleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IsolationBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for IsolationBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One isolation bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBundle {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub tags: Vec<String>,
    pub segment: String,
    pub archived: bool,
    pub contact_email: String,
    pub timezone: String,
    pub channel: String,
    pub phone: String,
    pub status: IsolationBundleStatus,
}

impl IsolationBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationBundleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationBundleStatus) -> bool {
        match self.status {
            IsolationBundleStatus::Planned => {
                match next {
                    IsolationBundleStatus::Completed | IsolationBundleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationBundleStatus::Scheduled => {
                return next == IsolationBundleStatus::Cancelled;
            }
            IsolationBundleStatus::InProgress => {
                return next == IsolationBundleStatus::Cancelled;
            }
            IsolationBundleStatus::Completed => {
                return next == IsolationBundleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation bundle {} cannot move from {} to {}",
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

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> IsolationBundle {
        IsolationBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            tags: self.tags.clone(),
            segment: self.segment.clone(),
            archived: self.archived,
            contact_email: self.contact_email.clone(),
            timezone: self.timezone.clone(),
            channel: self.channel.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a isolation bundle in status planned.
pub fn new_isolation_bundle(code: &str, title: &str) -> IsolationBundle {
    IsolationBundle {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        tags: Vec::new(),
        segment: "".to_string(),
        archived: false,
        contact_email: "".to_string(),
        timezone: "".to_string(),
        channel: "".to_string(),
        phone: "".to_string(),
        status: IsolationBundleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_isolation_bundle("code-433", "title-492");
        assert_eq!(item.status, IsolationBundleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_isolation_bundle("code-433", "title-492");
        if item.move_to(IsolationBundleStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationBundleStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_isolation_bundle("code-433", "title-492");
        item.status = IsolationBundleStatus::Cancelled;
        assert!(
            item.move_to(IsolationBundleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
