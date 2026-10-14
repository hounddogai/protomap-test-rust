use super::*;

use acme_kit::text;

/// Where a nonce summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceSummaryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl NonceSummaryStatus {
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

impl std::fmt::Display for NonceSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a nonce summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceSummary {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub floor_value: f64,
    pub issuer: String,
    pub checksum: String,
    pub status: NonceSummaryStatus,
}

impl NonceSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceSummaryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceSummaryStatus) -> bool {
        match self.status {
            NonceSummaryStatus::Planned => {
                match next {
                    NonceSummaryStatus::Completed | NonceSummaryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceSummaryStatus::Scheduled => {
                match next {
                    NonceSummaryStatus::InProgress | NonceSummaryStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceSummaryStatus::InProgress => {
                return next == NonceSummaryStatus::Completed;
            }
            NonceSummaryStatus::Completed => {
                match next {
                    NonceSummaryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: NonceSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "nonce summary", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.window_days as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceSummary {
        NonceSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            floor_value: self.floor_value,
            issuer: self.issuer.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a nonce summary in status planned.
pub fn new_nonce_summary(code: &str, title: &str) -> NonceSummary {
    NonceSummary {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        floor_value: 0.0,
        issuer: "".to_string(),
        checksum: "".to_string(),
        status: NonceSummaryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_nonce_summary("code-132", "title-817");
        assert_eq!(item.status, NonceSummaryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_nonce_summary("code-132", "title-817");
        if item.move_to(NonceSummaryStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceSummaryStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_nonce_summary("code-132", "title-817");
        item.status = NonceSummaryStatus::Cancelled;
        assert!(
            item.move_to(NonceSummaryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
