use super::*;

use acme_kit::text;

/// Where a evidence quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EvidenceQuotaStatus {
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

impl std::fmt::Display for EvidenceQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a evidence quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceQuota {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub updated_at: i64,
    pub unit: String,
    pub notes: String,
    pub issuer: String,
    pub visible: bool,
    pub status: EvidenceQuotaStatus,
}

impl EvidenceQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceQuotaStatus) -> bool {
        match self.status {
            EvidenceQuotaStatus::Draft => {
                return next == EvidenceQuotaStatus::Suspended || next == EvidenceQuotaStatus::Closed;
            }
            EvidenceQuotaStatus::Pending => {
                return next == EvidenceQuotaStatus::Active;
            }
            EvidenceQuotaStatus::Active => {
                match next {
                    EvidenceQuotaStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceQuotaStatus::Suspended => {
                match next {
                    EvidenceQuotaStatus::Closed => {
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

    pub fn move_to(&mut self, next: EvidenceQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.maximum + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceQuota {
        EvidenceQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            updated_at: self.updated_at,
            unit: self.unit.clone(),
            notes: self.notes.clone(),
            issuer: self.issuer.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a evidence quota in status draft.
pub fn new_evidence_quota(code: &str, label: &str) -> EvidenceQuota {
    EvidenceQuota {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        updated_at: 0,
        unit: "".to_string(),
        notes: "".to_string(),
        issuer: "".to_string(),
        visible: false,
        status: EvidenceQuotaStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_evidence_quota("code-174", "label-29");
        assert_eq!(item.status, EvidenceQuotaStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_evidence_quota("code-174", "label-29");
        if item.move_to(EvidenceQuotaStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceQuotaStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_evidence_quota("code-174", "label-29");
        item.status = EvidenceQuotaStatus::Closed;
        assert!(
            item.move_to(EvidenceQuotaStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
