use super::*;

use acme_kit::text;

/// Where a regional verifier batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalVerifierBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalVerifierBatchStatus {
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

impl std::fmt::Display for RegionalVerifierBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional verifier batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierBatch {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub ceiling: f64,
    pub region: String,
    pub margin: f64,
    pub principal: String,
    pub status: RegionalVerifierBatchStatus,
}

impl RegionalVerifierBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalVerifierBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalVerifierBatchStatus) -> bool {
        match self.status {
            RegionalVerifierBatchStatus::New => {
                return next == RegionalVerifierBatchStatus::Done || next == RegionalVerifierBatchStatus::Failed;
            }
            RegionalVerifierBatchStatus::Queued => {
                return next == RegionalVerifierBatchStatus::Running || next == RegionalVerifierBatchStatus::Done;
            }
            RegionalVerifierBatchStatus::Running => {
                match next {
                    RegionalVerifierBatchStatus::Done | RegionalVerifierBatchStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalVerifierBatchStatus::Done => {
                return next == RegionalVerifierBatchStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalVerifierBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional verifier batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.margin - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalVerifierBatch {
        RegionalVerifierBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            ceiling: self.ceiling,
            region: self.region.clone(),
            margin: self.margin,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional verifier batch in status new.
pub fn new_regional_verifier_batch(code: &str, label: &str) -> RegionalVerifierBatch {
    RegionalVerifierBatch {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        ceiling: 0.0,
        region: "".to_string(),
        margin: 0.0,
        principal: "".to_string(),
        status: RegionalVerifierBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_verifier_batch("code-423", "label-949");
        assert_eq!(item.status, RegionalVerifierBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_regional_verifier_batch("code-423", "label-949");
        if item.move_to(RegionalVerifierBatchStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalVerifierBatchStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_verifier_batch("code-423", "label-949");
        item.status = RegionalVerifierBatchStatus::Failed;
        assert!(
            item.move_to(RegionalVerifierBatchStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
