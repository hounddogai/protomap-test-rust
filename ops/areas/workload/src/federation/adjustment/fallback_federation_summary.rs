use super::*;

use acme_kit::text;

/// Where a fallback federation summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackFederationSummaryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackFederationSummaryStatus {
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

impl std::fmt::Display for FallbackFederationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback federation summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFederationSummary {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub contact_email: String,
    pub threshold: f64,
    pub budget: f64,
    pub rate: f64,
    pub updated_at: i64,
    pub status: FallbackFederationSummaryStatus,
}

impl FallbackFederationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackFederationSummaryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackFederationSummaryStatus) -> bool {
        match self.status {
            FallbackFederationSummaryStatus::New => {
                match next {
                    FallbackFederationSummaryStatus::Running | FallbackFederationSummaryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFederationSummaryStatus::Queued => {
                match next {
                    FallbackFederationSummaryStatus::Running | FallbackFederationSummaryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFederationSummaryStatus::Running => {
                match next {
                    FallbackFederationSummaryStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFederationSummaryStatus::Done => {
                match next {
                    FallbackFederationSummaryStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackFederationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback federation summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.expires_at as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackFederationSummary {
        FallbackFederationSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            contact_email: self.contact_email.clone(),
            threshold: self.threshold,
            budget: self.budget,
            rate: self.rate,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback federation summary in status new.
pub fn new_fallback_federation_summary(code: &str, label: &str) -> FallbackFederationSummary {
    FallbackFederationSummary {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        contact_email: "".to_string(),
        threshold: 0.0,
        budget: 0.0,
        rate: 0.0,
        updated_at: 0,
        status: FallbackFederationSummaryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_federation_summary("code-123", "label-938");
        assert_eq!(item.status, FallbackFederationSummaryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_fallback_federation_summary("code-123", "label-938");
        if item.move_to(FallbackFederationSummaryStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackFederationSummaryStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_federation_summary("code-123", "label-938");
        item.status = FallbackFederationSummaryStatus::Failed;
        assert!(
            item.move_to(FallbackFederationSummaryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
