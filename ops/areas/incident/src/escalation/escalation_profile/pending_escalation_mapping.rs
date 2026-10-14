use super::*;

use acme_kit::text;

/// Where a pending escalation mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingEscalationMappingStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingEscalationMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PendingEscalationMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending escalation mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationMapping {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub cipher: String,
    pub sequence: i64,
    pub description: String,
    pub created_at: i64,
    pub limit: i64,
    pub summary: String,
    pub status: PendingEscalationMappingStatus,
}

impl PendingEscalationMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingEscalationMappingStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingEscalationMappingStatus) -> bool {
        match self.status {
            PendingEscalationMappingStatus::Draft => {
                match next {
                    PendingEscalationMappingStatus::Pending | PendingEscalationMappingStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingEscalationMappingStatus::Pending => {
                match next {
                    PendingEscalationMappingStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingEscalationMappingStatus::Active => {
                match next {
                    PendingEscalationMappingStatus::Suspended | PendingEscalationMappingStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingEscalationMappingStatus::Suspended => {
                return next == PendingEscalationMappingStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingEscalationMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending escalation mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.created_at as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingEscalationMapping {
        PendingEscalationMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            cipher: self.cipher.clone(),
            sequence: self.sequence,
            description: self.description.clone(),
            created_at: self.created_at,
            limit: self.limit,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending escalation mapping in status draft.
pub fn new_pending_escalation_mapping(code: &str, title: &str) -> PendingEscalationMapping {
    PendingEscalationMapping {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        cipher: "".to_string(),
        sequence: 0,
        description: "".to_string(),
        created_at: 0,
        limit: 0,
        summary: "".to_string(),
        status: PendingEscalationMappingStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_escalation_mapping("code-298", "title-943");
        assert_eq!(item.status, PendingEscalationMappingStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_pending_escalation_mapping("code-298", "title-943");
        if item.move_to(PendingEscalationMappingStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingEscalationMappingStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_escalation_mapping("code-298", "title-943");
        item.status = PendingEscalationMappingStatus::Closed;
        assert!(
            item.move_to(PendingEscalationMappingStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
