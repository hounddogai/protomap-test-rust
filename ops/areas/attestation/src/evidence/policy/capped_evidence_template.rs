use super::*;

use acme_kit::text;

/// Where a capped evidence template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedEvidenceTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedEvidenceTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for CappedEvidenceTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped evidence template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceTemplate {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub issuer: String,
    pub capacity: i64,
    pub country: String,
    pub duration_minutes: i64,
    pub expires_at: i64,
    pub status: CappedEvidenceTemplateStatus,
}

impl CappedEvidenceTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedEvidenceTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedEvidenceTemplateStatus) -> bool {
        match self.status {
            CappedEvidenceTemplateStatus::Open => {
                match next {
                    CappedEvidenceTemplateStatus::Resolved | CappedEvidenceTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedEvidenceTemplateStatus::InReview => {
                match next {
                    CappedEvidenceTemplateStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedEvidenceTemplateStatus::Resolved => {
                return next == CappedEvidenceTemplateStatus::Reopened
                    || next == CappedEvidenceTemplateStatus::Archived;
            }
            CappedEvidenceTemplateStatus::Reopened => {
                return next == CappedEvidenceTemplateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedEvidenceTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped evidence template {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "capped evidence template", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.capacity as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedEvidenceTemplate {
        CappedEvidenceTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            issuer: self.issuer.clone(),
            capacity: self.capacity,
            country: self.country.clone(),
            duration_minutes: self.duration_minutes,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a capped evidence template in status open.
pub fn new_capped_evidence_template(code: &str, title: &str) -> CappedEvidenceTemplate {
    CappedEvidenceTemplate {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        issuer: "".to_string(),
        capacity: 0,
        country: "".to_string(),
        duration_minutes: 0,
        expires_at: 0,
        status: CappedEvidenceTemplateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_evidence_template("code-71", "title-413");
        assert_eq!(item.status, CappedEvidenceTemplateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_capped_evidence_template("code-71", "title-413");
        if item.move_to(CappedEvidenceTemplateStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedEvidenceTemplateStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_evidence_template("code-71", "title-413");
        item.status = CappedEvidenceTemplateStatus::Archived;
        assert!(
            item.move_to(CappedEvidenceTemplateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
