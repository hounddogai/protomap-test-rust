use super::*;

use acme_kit::text;

/// Where a verifier batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierBatchStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl VerifierBatchStatus {
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

impl std::fmt::Display for VerifierBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a verifier batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBatch {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub margin: f64,
    pub budget: f64,
    pub expires_at: i64,
    pub audience: String,
    pub notes: String,
    pub checksum: String,
    pub unit: String,
    pub source: String,
    pub status: VerifierBatchStatus,
}

impl VerifierBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierBatchStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierBatchStatus) -> bool {
        match self.status {
            VerifierBatchStatus::Draft => {
                match next {
                    VerifierBatchStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierBatchStatus::Pending => {
                return next == VerifierBatchStatus::Suspended || next == VerifierBatchStatus::Closed;
            }
            VerifierBatchStatus::Active => {
                match next {
                    VerifierBatchStatus::Suspended | VerifierBatchStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierBatchStatus::Suspended => {
                match next {
                    VerifierBatchStatus::Closed => {
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

    pub fn move_to(&mut self, next: VerifierBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier batch {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        (self.budget * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierBatch {
        VerifierBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            margin: self.margin,
            budget: self.budget,
            expires_at: self.expires_at,
            audience: self.audience.clone(),
            notes: self.notes.clone(),
            checksum: self.checksum.clone(),
            unit: self.unit.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a verifier batch in status draft.
pub fn new_verifier_batch(code: &str, label: &str) -> VerifierBatch {
    VerifierBatch {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        margin: 0.0,
        budget: 0.0,
        expires_at: 0,
        audience: "".to_string(),
        notes: "".to_string(),
        checksum: "".to_string(),
        unit: "".to_string(),
        source: "".to_string(),
        status: VerifierBatchStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_verifier_batch("code-951", "label-209");
        assert_eq!(item.status, VerifierBatchStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_verifier_batch("code-951", "label-209");
        if item.move_to(VerifierBatchStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierBatchStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_verifier_batch("code-951", "label-209");
        item.status = VerifierBatchStatus::Closed;
        assert!(
            item.move_to(VerifierBatchStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
