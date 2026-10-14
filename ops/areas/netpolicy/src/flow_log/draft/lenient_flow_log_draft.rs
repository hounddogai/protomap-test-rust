use super::*;

use acme_kit::text;

/// Where a lenient flow log draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientFlowLogDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientFlowLogDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientFlowLogDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient flow log draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogDraft {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub retries: i64,
    pub revision: i64,
    pub archived: bool,
    pub grace_days: i64,
    pub port: i64,
    pub notes: String,
    pub phone: String,
    pub status: LenientFlowLogDraftStatus,
}

impl LenientFlowLogDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientFlowLogDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientFlowLogDraftStatus) -> bool {
        match self.status {
            LenientFlowLogDraftStatus::Created => {
                match next {
                    LenientFlowLogDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFlowLogDraftStatus::Validated => {
                return next == LenientFlowLogDraftStatus::Retired;
            }
            LenientFlowLogDraftStatus::Published => {
                return next == LenientFlowLogDraftStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientFlowLogDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient flow log draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient flow log draft", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.grace_days as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientFlowLogDraft {
        LenientFlowLogDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            retries: self.retries,
            revision: self.revision,
            archived: self.archived,
            grace_days: self.grace_days,
            port: self.port,
            notes: self.notes.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient flow log draft in status created.
pub fn new_lenient_flow_log_draft(code: &str, name: &str) -> LenientFlowLogDraft {
    LenientFlowLogDraft {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        retries: 0,
        revision: 0,
        archived: false,
        grace_days: 0,
        port: 0,
        notes: "".to_string(),
        phone: "".to_string(),
        status: LenientFlowLogDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_flow_log_draft("code-463", "name-587");
        assert_eq!(item.status, LenientFlowLogDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_lenient_flow_log_draft("code-463", "name-587");
        if item.move_to(LenientFlowLogDraftStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientFlowLogDraftStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_flow_log_draft("code-463", "name-587");
        item.status = LenientFlowLogDraftStatus::Retired;
        assert!(
            item.move_to(LenientFlowLogDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
