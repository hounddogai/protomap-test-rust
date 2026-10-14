use super::*;

use acme_kit::text;

/// Where a escalation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EscalationTemplateStatus {
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

impl std::fmt::Display for EscalationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTemplate {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub minimum: f64,
    pub burst: i64,
    pub grace_days: i64,
    pub rotation_days: i64,
    pub external_ref: String,
    pub starts_at: i64,
    pub created_at: i64,
    pub summary: String,
    pub status: EscalationTemplateStatus,
}

impl EscalationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationTemplateStatus) -> bool {
        match self.status {
            EscalationTemplateStatus::Open => {
                match next {
                    EscalationTemplateStatus::InReview | EscalationTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationTemplateStatus::InReview => {
                match next {
                    EscalationTemplateStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationTemplateStatus::Resolved => {
                return next == EscalationTemplateStatus::Reopened || next == EscalationTemplateStatus::Archived;
            }
            EscalationTemplateStatus::Reopened => {
                match next {
                    EscalationTemplateStatus::Archived => {
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

    pub fn move_to(&mut self, next: EscalationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.minimum - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationTemplate {
        EscalationTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            minimum: self.minimum,
            burst: self.burst,
            grace_days: self.grace_days,
            rotation_days: self.rotation_days,
            external_ref: self.external_ref.clone(),
            starts_at: self.starts_at,
            created_at: self.created_at,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a escalation template in status open.
pub fn new_escalation_template(code: &str, name: &str) -> EscalationTemplate {
    EscalationTemplate {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        minimum: 0.0,
        burst: 0,
        grace_days: 0,
        rotation_days: 0,
        external_ref: "".to_string(),
        starts_at: 0,
        created_at: 0,
        summary: "".to_string(),
        status: EscalationTemplateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_escalation_template("code-328", "name-401");
        assert_eq!(item.status, EscalationTemplateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_escalation_template("code-328", "name-401");
        if item.move_to(EscalationTemplateStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationTemplateStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_escalation_template("code-328", "name-401");
        item.status = EscalationTemplateStatus::Archived;
        assert!(
            item.move_to(EscalationTemplateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
