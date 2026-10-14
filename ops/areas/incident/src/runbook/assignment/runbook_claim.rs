use super::*;

use acme_kit::text;

/// Where a runbook claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookClaimStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RunbookClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RunbookClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One runbook claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookClaim {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub version: i64,
    pub window_days: i64,
    pub revision: i64,
    pub segment: String,
    pub timezone: String,
    pub weight: f64,
    pub currency: String,
    pub sequence: i64,
    pub status: RunbookClaimStatus,
}

impl RunbookClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookClaimStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookClaimStatus) -> bool {
        match self.status {
            RunbookClaimStatus::Created => {
                return next == RunbookClaimStatus::Validated;
            }
            RunbookClaimStatus::Validated => {
                return next == RunbookClaimStatus::Published;
            }
            RunbookClaimStatus::Published => {
                return next == RunbookClaimStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.sequence as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookClaim {
        RunbookClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            version: self.version,
            window_days: self.window_days,
            revision: self.revision,
            segment: self.segment.clone(),
            timezone: self.timezone.clone(),
            weight: self.weight,
            currency: self.currency.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a runbook claim in status created.
pub fn new_runbook_claim(code: &str, label: &str) -> RunbookClaim {
    RunbookClaim {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        version: 0,
        window_days: 0,
        revision: 0,
        segment: "".to_string(),
        timezone: "".to_string(),
        weight: 0.0,
        currency: "".to_string(),
        sequence: 0,
        status: RunbookClaimStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_runbook_claim("code-662", "label-42");
        assert_eq!(item.status, RunbookClaimStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_runbook_claim("code-662", "label-42");
        if item.move_to(RunbookClaimStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookClaimStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_runbook_claim("code-662", "label-42");
        item.status = RunbookClaimStatus::Retired;
        assert!(
            item.move_to(RunbookClaimStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
