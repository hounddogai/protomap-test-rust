use super::*;

use acme_kit::text;

/// Where a primary canary summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryCanarySummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryCanarySummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryCanarySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary canary summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanarySummary {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub region: String,
    pub revision: i64,
    pub audience: String,
    pub status: PrimaryCanarySummaryStatus,
}

impl PrimaryCanarySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryCanarySummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryCanarySummaryStatus) -> bool {
        match self.status {
            PrimaryCanarySummaryStatus::Created => {
                return next == PrimaryCanarySummaryStatus::Validated;
            }
            PrimaryCanarySummaryStatus::Validated => {
                return next == PrimaryCanarySummaryStatus::Published || next == PrimaryCanarySummaryStatus::Retired;
            }
            PrimaryCanarySummaryStatus::Published => {
                match next {
                    PrimaryCanarySummaryStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimaryCanarySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary canary summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> PrimaryCanarySummary {
        PrimaryCanarySummary {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            region: self.region.clone(),
            revision: self.revision,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary canary summary in status created.
pub fn new_primary_canary_summary(code: &str, label: &str) -> PrimaryCanarySummary {
    PrimaryCanarySummary {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        region: "".to_string(),
        revision: 0,
        audience: "".to_string(),
        status: PrimaryCanarySummaryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_primary_canary_summary("code-605", "label-764");
        assert_eq!(item.status, PrimaryCanarySummaryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_primary_canary_summary("code-605", "label-764");
        if item.move_to(PrimaryCanarySummaryStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryCanarySummaryStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_primary_canary_summary("code-605", "label-764");
        item.status = PrimaryCanarySummaryStatus::Retired;
        assert!(
            item.move_to(PrimaryCanarySummaryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
