use super::*;

use acme_kit::text;

/// Where a batched federation summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedFederationSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedFederationSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for BatchedFederationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched federation summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationSummary {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub budget: f64,
    pub region: String,
    pub floor_value: f64,
    pub rate: f64,
    pub archived: bool,
    pub threshold: f64,
    pub status: BatchedFederationSummaryStatus,
}

impl BatchedFederationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedFederationSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedFederationSummaryStatus) -> bool {
        match self.status {
            BatchedFederationSummaryStatus::Draft => {
                match next {
                    BatchedFederationSummaryStatus::Pending | BatchedFederationSummaryStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFederationSummaryStatus::Pending => {
                return next == BatchedFederationSummaryStatus::Suspended
                    || next == BatchedFederationSummaryStatus::Closed;
            }
            BatchedFederationSummaryStatus::Active => {
                match next {
                    BatchedFederationSummaryStatus::Suspended | BatchedFederationSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFederationSummaryStatus::Suspended => {
                match next {
                    BatchedFederationSummaryStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedFederationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched federation summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched federation summary", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.minimum - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedFederationSummary {
        BatchedFederationSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            budget: self.budget,
            region: self.region.clone(),
            floor_value: self.floor_value,
            rate: self.rate,
            archived: self.archived,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a batched federation summary in status draft.
pub fn new_batched_federation_summary(code: &str, name: &str) -> BatchedFederationSummary {
    BatchedFederationSummary {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        budget: 0.0,
        region: "".to_string(),
        floor_value: 0.0,
        rate: 0.0,
        archived: false,
        threshold: 0.0,
        status: BatchedFederationSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_federation_summary("code-345", "name-769");
        assert_eq!(item.status, BatchedFederationSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_batched_federation_summary("code-345", "name-769");
        if item.move_to(BatchedFederationSummaryStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedFederationSummaryStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_federation_summary("code-345", "name-769");
        item.status = BatchedFederationSummaryStatus::Closed;
        assert!(
            item.move_to(BatchedFederationSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
