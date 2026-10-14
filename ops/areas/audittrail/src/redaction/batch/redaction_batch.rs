use super::*;

use acme_kit::text;

/// Where a redaction batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionBatchStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RedactionBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RedactionBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBatch {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub cluster: String,
    pub name: String,
    pub external_ref: String,
    pub minimum: f64,
    pub version: i64,
    pub rate: f64,
    pub description: String,
    pub tenant_id: String,
    pub status: RedactionBatchStatus,
}

impl RedactionBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionBatchStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionBatchStatus) -> bool {
        match self.status {
            RedactionBatchStatus::Created => {
                return next == RedactionBatchStatus::Validated || next == RedactionBatchStatus::Retired;
            }
            RedactionBatchStatus::Validated => {
                return next == RedactionBatchStatus::Published || next == RedactionBatchStatus::Retired;
            }
            RedactionBatchStatus::Published => {
                return next == RedactionBatchStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.rate + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionBatch {
        RedactionBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            cluster: self.cluster.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            minimum: self.minimum,
            version: self.version,
            rate: self.rate,
            description: self.description.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a redaction batch in status created.
pub fn new_redaction_batch(code: &str, label: &str) -> RedactionBatch {
    RedactionBatch {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        cluster: "".to_string(),
        name: "".to_string(),
        external_ref: "".to_string(),
        minimum: 0.0,
        version: 0,
        rate: 0.0,
        description: "".to_string(),
        tenant_id: "".to_string(),
        status: RedactionBatchStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_redaction_batch("code-525", "label-70");
        assert_eq!(item.status, RedactionBatchStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_redaction_batch("code-525", "label-70");
        if item.move_to(RedactionBatchStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionBatchStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_redaction_batch("code-525", "label-70");
        item.status = RedactionBatchStatus::Retired;
        assert!(
            item.move_to(RedactionBatchStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
