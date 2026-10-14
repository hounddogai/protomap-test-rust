use super::*;

use acme_kit::text;

/// Where a manual evidence template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualEvidenceTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualEvidenceTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualEvidenceTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual evidence template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceTemplate {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub tenant_id: String,
    pub ratio: f64,
    pub grace_days: i64,
    pub margin: f64,
    pub status: ManualEvidenceTemplateStatus,
}

impl ManualEvidenceTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualEvidenceTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualEvidenceTemplateStatus) -> bool {
        match self.status {
            ManualEvidenceTemplateStatus::Proposed => {
                match next {
                    ManualEvidenceTemplateStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualEvidenceTemplateStatus::Approved => {
                return next == ManualEvidenceTemplateStatus::Rejected
                    || next == ManualEvidenceTemplateStatus::Withdrawn;
            }
            ManualEvidenceTemplateStatus::Rejected => {
                return next == ManualEvidenceTemplateStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualEvidenceTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual evidence template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.margin + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualEvidenceTemplate {
        ManualEvidenceTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            tenant_id: self.tenant_id.clone(),
            ratio: self.ratio,
            grace_days: self.grace_days,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual evidence template in status proposed.
pub fn new_manual_evidence_template(code: &str, name: &str) -> ManualEvidenceTemplate {
    ManualEvidenceTemplate {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        tenant_id: "".to_string(),
        ratio: 0.0,
        grace_days: 0,
        margin: 0.0,
        status: ManualEvidenceTemplateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_evidence_template("code-882", "name-65");
        assert_eq!(item.status, ManualEvidenceTemplateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_manual_evidence_template("code-882", "name-65");
        if item.move_to(ManualEvidenceTemplateStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualEvidenceTemplateStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_evidence_template("code-882", "name-65");
        item.status = ManualEvidenceTemplateStatus::Withdrawn;
        assert!(
            item.move_to(ManualEvidenceTemplateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
