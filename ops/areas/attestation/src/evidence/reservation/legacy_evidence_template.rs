use super::*;

use acme_kit::text;

/// Where a legacy evidence template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyEvidenceTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyEvidenceTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyEvidenceTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy evidence template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceTemplate {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub contact_email: String,
    pub capacity: i64,
    pub audience: String,
    pub description: String,
    pub archived: bool,
    pub host: String,
    pub rate: f64,
    pub channel: String,
    pub status: LegacyEvidenceTemplateStatus,
}

impl LegacyEvidenceTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyEvidenceTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyEvidenceTemplateStatus) -> bool {
        match self.status {
            LegacyEvidenceTemplateStatus::Proposed => {
                match next {
                    LegacyEvidenceTemplateStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEvidenceTemplateStatus::Approved => {
                return next == LegacyEvidenceTemplateStatus::Rejected
                    || next == LegacyEvidenceTemplateStatus::Withdrawn;
            }
            LegacyEvidenceTemplateStatus::Rejected => {
                match next {
                    LegacyEvidenceTemplateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LegacyEvidenceTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy evidence template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.capacity as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyEvidenceTemplate {
        LegacyEvidenceTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            contact_email: self.contact_email.clone(),
            capacity: self.capacity,
            audience: self.audience.clone(),
            description: self.description.clone(),
            archived: self.archived,
            host: self.host.clone(),
            rate: self.rate,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a legacy evidence template in status proposed.
pub fn new_legacy_evidence_template(code: &str, label: &str) -> LegacyEvidenceTemplate {
    LegacyEvidenceTemplate {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        contact_email: "".to_string(),
        capacity: 0,
        audience: "".to_string(),
        description: "".to_string(),
        archived: false,
        host: "".to_string(),
        rate: 0.0,
        channel: "".to_string(),
        status: LegacyEvidenceTemplateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_legacy_evidence_template("code-469", "label-669");
        assert_eq!(item.status, LegacyEvidenceTemplateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_legacy_evidence_template("code-469", "label-669");
        if item.move_to(LegacyEvidenceTemplateStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyEvidenceTemplateStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_legacy_evidence_template("code-469", "label-669");
        item.status = LegacyEvidenceTemplateStatus::Withdrawn;
        assert!(
            item.move_to(LegacyEvidenceTemplateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
