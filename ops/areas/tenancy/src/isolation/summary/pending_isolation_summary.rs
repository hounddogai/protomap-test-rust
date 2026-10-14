use super::*;

use acme_kit::text;

/// Where a pending isolation summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingIsolationSummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingIsolationSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingIsolationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending isolation summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationSummary {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub score: f64,
    pub cipher: String,
    pub minimum: f64,
    pub status: PendingIsolationSummaryStatus,
}

impl PendingIsolationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingIsolationSummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingIsolationSummaryStatus) -> bool {
        match self.status {
            PendingIsolationSummaryStatus::Created => {
                match next {
                    PendingIsolationSummaryStatus::Published | PendingIsolationSummaryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingIsolationSummaryStatus::Validated => {
                return next == PendingIsolationSummaryStatus::Published;
            }
            PendingIsolationSummaryStatus::Published => {
                match next {
                    PendingIsolationSummaryStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingIsolationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending isolation summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.score + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingIsolationSummary {
        PendingIsolationSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            score: self.score,
            cipher: self.cipher.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending isolation summary in status created.
pub fn new_pending_isolation_summary(code: &str, label: &str) -> PendingIsolationSummary {
    PendingIsolationSummary {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        score: 0.0,
        cipher: "".to_string(),
        minimum: 0.0,
        status: PendingIsolationSummaryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_isolation_summary("code-730", "label-770");
        assert_eq!(item.status, PendingIsolationSummaryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_pending_isolation_summary("code-730", "label-770");
        if item.move_to(PendingIsolationSummaryStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingIsolationSummaryStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_isolation_summary("code-730", "label-770");
        item.status = PendingIsolationSummaryStatus::Retired;
        assert!(
            item.move_to(PendingIsolationSummaryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
