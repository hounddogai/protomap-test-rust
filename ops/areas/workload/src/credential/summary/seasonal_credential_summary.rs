use super::*;

use acme_kit::text;

/// Where a seasonal credential summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalCredentialSummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalCredentialSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalCredentialSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal credential summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSummary {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub amount: f64,
    pub ends_at: i64,
    pub principal: String,
    pub batch_size: i64,
    pub quota_units: i64,
    pub cluster: String,
    pub reviewer: String,
    pub status: SeasonalCredentialSummaryStatus,
}

impl SeasonalCredentialSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalCredentialSummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalCredentialSummaryStatus) -> bool {
        match self.status {
            SeasonalCredentialSummaryStatus::Created => {
                match next {
                    SeasonalCredentialSummaryStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalCredentialSummaryStatus::Validated => {
                match next {
                    SeasonalCredentialSummaryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalCredentialSummaryStatus::Published => {
                match next {
                    SeasonalCredentialSummaryStatus::Retired => {
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

    pub fn move_to(&mut self, next: SeasonalCredentialSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal credential summary {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ends_at as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalCredentialSummary {
        SeasonalCredentialSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            amount: self.amount,
            ends_at: self.ends_at,
            principal: self.principal.clone(),
            batch_size: self.batch_size,
            quota_units: self.quota_units,
            cluster: self.cluster.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a seasonal credential summary in status created.
pub fn new_seasonal_credential_summary(code: &str, title: &str) -> SeasonalCredentialSummary {
    SeasonalCredentialSummary {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        amount: 0.0,
        ends_at: 0,
        principal: "".to_string(),
        batch_size: 0,
        quota_units: 0,
        cluster: "".to_string(),
        reviewer: "".to_string(),
        status: SeasonalCredentialSummaryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_seasonal_credential_summary("code-781", "title-217");
        assert_eq!(item.status, SeasonalCredentialSummaryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_seasonal_credential_summary("code-781", "title-217");
        if item.move_to(SeasonalCredentialSummaryStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalCredentialSummaryStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_seasonal_credential_summary("code-781", "title-217");
        item.status = SeasonalCredentialSummaryStatus::Retired;
        assert!(
            item.move_to(SeasonalCredentialSummaryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
