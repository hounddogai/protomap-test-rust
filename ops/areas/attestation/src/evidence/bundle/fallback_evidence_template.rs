use super::*;

use acme_kit::text;

/// Where a fallback evidence template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackEvidenceTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackEvidenceTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FallbackEvidenceTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback evidence template as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceTemplate {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub owner_id: String,
    pub description: String,
    pub status: FallbackEvidenceTemplateStatus,
}

impl FallbackEvidenceTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackEvidenceTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackEvidenceTemplateStatus) -> bool {
        match self.status {
            FallbackEvidenceTemplateStatus::Planned => {
                match next {
                    FallbackEvidenceTemplateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEvidenceTemplateStatus::Scheduled => {
                return next == FallbackEvidenceTemplateStatus::InProgress
                    || next == FallbackEvidenceTemplateStatus::Cancelled;
            }
            FallbackEvidenceTemplateStatus::InProgress => {
                match next {
                    FallbackEvidenceTemplateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEvidenceTemplateStatus::Completed => {
                match next {
                    FallbackEvidenceTemplateStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FallbackEvidenceTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback evidence template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> FallbackEvidenceTemplate {
        FallbackEvidenceTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            owner_id: self.owner_id.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback evidence template in status planned.
pub fn new_fallback_evidence_template(code: &str, title: &str) -> FallbackEvidenceTemplate {
    FallbackEvidenceTemplate {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        owner_id: "".to_string(),
        description: "".to_string(),
        status: FallbackEvidenceTemplateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_evidence_template("code-620", "title-12");
        assert_eq!(item.status, FallbackEvidenceTemplateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_fallback_evidence_template("code-620", "title-12");
        if item.move_to(FallbackEvidenceTemplateStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackEvidenceTemplateStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_evidence_template("code-620", "title-12");
        item.status = FallbackEvidenceTemplateStatus::Cancelled;
        assert!(
            item.move_to(FallbackEvidenceTemplateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
