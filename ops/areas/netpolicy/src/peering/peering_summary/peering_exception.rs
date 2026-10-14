use super::*;

use acme_kit::text;

/// Where a peering exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PeeringExceptionStatus {
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

impl std::fmt::Display for PeeringExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringException {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub summary: String,
    pub port: i64,
    pub revision: i64,
    pub notes: String,
    pub updated_at: i64,
    pub source: String,
    pub category: String,
    pub status: PeeringExceptionStatus,
}

impl PeeringException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringExceptionStatus) -> bool {
        match self.status {
            PeeringExceptionStatus::Draft => {
                return next == PeeringExceptionStatus::Active || next == PeeringExceptionStatus::Closed;
            }
            PeeringExceptionStatus::Pending => {
                return next == PeeringExceptionStatus::Active || next == PeeringExceptionStatus::Suspended;
            }
            PeeringExceptionStatus::Active => {
                match next {
                    PeeringExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringExceptionStatus::Suspended => {
                match next {
                    PeeringExceptionStatus::Closed => {
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

    pub fn move_to(&mut self, next: PeeringExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.revision as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringException {
        PeeringException {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            summary: self.summary.clone(),
            port: self.port,
            revision: self.revision,
            notes: self.notes.clone(),
            updated_at: self.updated_at,
            source: self.source.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a peering exception in status draft.
pub fn new_peering_exception(code: &str, label: &str) -> PeeringException {
    PeeringException {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        summary: "".to_string(),
        port: 0,
        revision: 0,
        notes: "".to_string(),
        updated_at: 0,
        source: "".to_string(),
        category: "".to_string(),
        status: PeeringExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_peering_exception("code-871", "label-218");
        assert_eq!(item.status, PeeringExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_peering_exception("code-871", "label-218");
        if item.move_to(PeeringExceptionStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringExceptionStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_peering_exception("code-871", "label-218");
        item.status = PeeringExceptionStatus::Closed;
        assert!(
            item.move_to(PeeringExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
