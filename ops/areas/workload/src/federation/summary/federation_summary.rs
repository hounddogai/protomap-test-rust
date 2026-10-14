use super::*;

use acme_kit::text;

/// Where a federation summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationSummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FederationSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FederationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationSummary {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub host: String,
    pub score: f64,
    pub owner_id: String,
    pub checksum: String,
    pub weight: f64,
    pub sequence: i64,
    pub status: FederationSummaryStatus,
}

impl FederationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationSummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationSummaryStatus) -> bool {
        match self.status {
            FederationSummaryStatus::Created => {
                match next {
                    FederationSummaryStatus::Validated | FederationSummaryStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationSummaryStatus::Validated => {
                return next == FederationSummaryStatus::Retired;
            }
            FederationSummaryStatus::Published => {
                match next {
                    FederationSummaryStatus::Retired => {
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

    pub fn move_to(&mut self, next: FederationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        (self.weight * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationSummary {
        FederationSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            host: self.host.clone(),
            score: self.score,
            owner_id: self.owner_id.clone(),
            checksum: self.checksum.clone(),
            weight: self.weight,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a federation summary in status created.
pub fn new_federation_summary(code: &str, title: &str) -> FederationSummary {
    FederationSummary {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        host: "".to_string(),
        score: 0.0,
        owner_id: "".to_string(),
        checksum: "".to_string(),
        weight: 0.0,
        sequence: 0,
        status: FederationSummaryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_federation_summary("code-594", "title-750");
        assert_eq!(item.status, FederationSummaryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_federation_summary("code-594", "title-750");
        if item.move_to(FederationSummaryStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationSummaryStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_federation_summary("code-594", "title-750");
        item.status = FederationSummaryStatus::Retired;
        assert!(
            item.move_to(FederationSummaryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
