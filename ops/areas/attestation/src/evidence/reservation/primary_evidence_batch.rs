use super::*;

use acme_kit::text;

/// Where a primary evidence batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEvidenceBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryEvidenceBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryEvidenceBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary evidence batch as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceBatch {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub principal: String,
    pub region: String,
    pub host: String,
    pub ratio: f64,
    pub pinned: bool,
    pub batch_size: i64,
    pub unit: String,
    pub status: PrimaryEvidenceBatchStatus,
}

impl PrimaryEvidenceBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEvidenceBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEvidenceBatchStatus) -> bool {
        match self.status {
            PrimaryEvidenceBatchStatus::Requested => {
                return next == PrimaryEvidenceBatchStatus::Revoked || next == PrimaryEvidenceBatchStatus::Expired;
            }
            PrimaryEvidenceBatchStatus::Granted => {
                match next {
                    PrimaryEvidenceBatchStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEvidenceBatchStatus::Revoked => {
                match next {
                    PrimaryEvidenceBatchStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryEvidenceBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary evidence batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.ratio + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryEvidenceBatch {
        PrimaryEvidenceBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            principal: self.principal.clone(),
            region: self.region.clone(),
            host: self.host.clone(),
            ratio: self.ratio,
            pinned: self.pinned,
            batch_size: self.batch_size,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary evidence batch in status requested.
pub fn new_primary_evidence_batch(code: &str, title: &str) -> PrimaryEvidenceBatch {
    PrimaryEvidenceBatch {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        principal: "".to_string(),
        region: "".to_string(),
        host: "".to_string(),
        ratio: 0.0,
        pinned: false,
        batch_size: 0,
        unit: "".to_string(),
        status: PrimaryEvidenceBatchStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_evidence_batch("code-12", "title-163");
        assert_eq!(item.status, PrimaryEvidenceBatchStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_primary_evidence_batch("code-12", "title-163");
        if item.move_to(PrimaryEvidenceBatchStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryEvidenceBatchStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_evidence_batch("code-12", "title-163");
        item.status = PrimaryEvidenceBatchStatus::Expired;
        assert!(
            item.move_to(PrimaryEvidenceBatchStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
