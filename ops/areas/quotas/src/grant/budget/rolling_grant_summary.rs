use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rolling grant summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingGrantSummaryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingGrantSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RollingGrantSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling grant summary as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantSummary {
    pub code: String,
    pub title: String,
    pub host: String,
    pub notes: String,
    pub attributes: BTreeMap<String, String>,
    pub amount: f64,
    pub status: RollingGrantSummaryStatus,
}

impl RollingGrantSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingGrantSummaryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingGrantSummaryStatus) -> bool {
        match self.status {
            RollingGrantSummaryStatus::New => {
                return next == RollingGrantSummaryStatus::Done || next == RollingGrantSummaryStatus::Failed;
            }
            RollingGrantSummaryStatus::Queued => {
                return next == RollingGrantSummaryStatus::Running;
            }
            RollingGrantSummaryStatus::Running => {
                match next {
                    RollingGrantSummaryStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingGrantSummaryStatus::Done => {
                return next == RollingGrantSummaryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingGrantSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling grant summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> RollingGrantSummary {
        RollingGrantSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            notes: self.notes.clone(),
            attributes: self.attributes.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling grant summary in status new.
pub fn new_rolling_grant_summary(code: &str, title: &str) -> RollingGrantSummary {
    RollingGrantSummary {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        notes: "".to_string(),
        attributes: BTreeMap::new(),
        amount: 0.0,
        status: RollingGrantSummaryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_grant_summary("code-562", "title-402");
        assert_eq!(item.status, RollingGrantSummaryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_rolling_grant_summary("code-562", "title-402");
        if item.move_to(RollingGrantSummaryStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingGrantSummaryStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_grant_summary("code-562", "title-402");
        item.status = RollingGrantSummaryStatus::Failed;
        assert!(
            item.move_to(RollingGrantSummaryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
