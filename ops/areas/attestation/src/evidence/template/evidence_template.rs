use super::*;

use acme_kit::text;

/// Where a evidence template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EvidenceTemplateStatus {
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

impl std::fmt::Display for EvidenceTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One evidence template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTemplate {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub owner_id: String,
    pub expires_at: i64,
    pub phone: String,
    pub reviewer: String,
    pub capacity: i64,
    pub status: EvidenceTemplateStatus,
}

impl EvidenceTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceTemplateStatus) -> bool {
        match self.status {
            EvidenceTemplateStatus::Planned => {
                return next == EvidenceTemplateStatus::InProgress || next == EvidenceTemplateStatus::Cancelled;
            }
            EvidenceTemplateStatus::Scheduled => {
                match next {
                    EvidenceTemplateStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceTemplateStatus::InProgress => {
                return next == EvidenceTemplateStatus::Completed;
            }
            EvidenceTemplateStatus::Completed => {
                return next == EvidenceTemplateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence template {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.expires_at as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceTemplate {
        EvidenceTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            owner_id: self.owner_id.clone(),
            expires_at: self.expires_at,
            phone: self.phone.clone(),
            reviewer: self.reviewer.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a evidence template in status planned.
pub fn new_evidence_template(code: &str, label: &str) -> EvidenceTemplate {
    EvidenceTemplate {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        owner_id: "".to_string(),
        expires_at: 0,
        phone: "".to_string(),
        reviewer: "".to_string(),
        capacity: 0,
        status: EvidenceTemplateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_evidence_template("code-638", "label-121");
        assert_eq!(item.status, EvidenceTemplateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_evidence_template("code-638", "label-121");
        if item.move_to(EvidenceTemplateStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceTemplateStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_evidence_template("code-638", "label-121");
        item.status = EvidenceTemplateStatus::Cancelled;
        assert!(
            item.move_to(EvidenceTemplateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
