use super::*;

use acme_kit::text;

/// Where a batched failover summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedFailoverSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedFailoverSummaryStatus {
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

impl std::fmt::Display for BatchedFailoverSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched failover summary as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverSummary {
    pub code: String,
    pub title: String,
    pub host: String,
    pub margin: f64,
    pub principal: String,
    pub threshold: f64,
    pub timezone: String,
    pub contact_email: String,
    pub fingerprint: String,
    pub issuer: String,
    pub status: BatchedFailoverSummaryStatus,
}

impl BatchedFailoverSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedFailoverSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedFailoverSummaryStatus) -> bool {
        match self.status {
            BatchedFailoverSummaryStatus::Draft => {
                match next {
                    BatchedFailoverSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFailoverSummaryStatus::Pending => {
                return next == BatchedFailoverSummaryStatus::Suspended || next == BatchedFailoverSummaryStatus::Closed;
            }
            BatchedFailoverSummaryStatus::Active => {
                match next {
                    BatchedFailoverSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFailoverSummaryStatus::Suspended => {
                match next {
                    BatchedFailoverSummaryStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedFailoverSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched failover summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched failover summary", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.threshold + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedFailoverSummary {
        BatchedFailoverSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            margin: self.margin,
            principal: self.principal.clone(),
            threshold: self.threshold,
            timezone: self.timezone.clone(),
            contact_email: self.contact_email.clone(),
            fingerprint: self.fingerprint.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched failover summary in status draft.
pub fn new_batched_failover_summary(code: &str, title: &str) -> BatchedFailoverSummary {
    BatchedFailoverSummary {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        margin: 0.0,
        principal: "".to_string(),
        threshold: 0.0,
        timezone: "".to_string(),
        contact_email: "".to_string(),
        fingerprint: "".to_string(),
        issuer: "".to_string(),
        status: BatchedFailoverSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_failover_summary("code-730", "title-394");
        assert_eq!(item.status, BatchedFailoverSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_batched_failover_summary("code-730", "title-394");
        if item.move_to(BatchedFailoverSummaryStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedFailoverSummaryStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_failover_summary("code-730", "title-394");
        item.status = BatchedFailoverSummaryStatus::Closed;
        assert!(
            item.move_to(BatchedFailoverSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
