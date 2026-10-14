use super::*;

use acme_kit::text;

/// Where a default query record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultQueryRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultQueryRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultQueryRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default query record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryRecord {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub title: String,
    pub minimum: f64,
    pub audience: String,
    pub segment: String,
    pub discount: f64,
    pub maximum: f64,
    pub source: String,
    pub external_ref: String,
    pub status: DefaultQueryRecordStatus,
}

impl DefaultQueryRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultQueryRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultQueryRecordStatus) -> bool {
        match self.status {
            DefaultQueryRecordStatus::Proposed => {
                return next == DefaultQueryRecordStatus::Rejected;
            }
            DefaultQueryRecordStatus::Approved => {
                match next {
                    DefaultQueryRecordStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultQueryRecordStatus::Rejected => {
                match next {
                    DefaultQueryRecordStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DefaultQueryRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default query record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default query record", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.maximum + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultQueryRecord {
        DefaultQueryRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            title: self.title.clone(),
            minimum: self.minimum,
            audience: self.audience.clone(),
            segment: self.segment.clone(),
            discount: self.discount,
            maximum: self.maximum,
            source: self.source.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a default query record in status proposed.
pub fn new_default_query_record(code: &str, label: &str) -> DefaultQueryRecord {
    DefaultQueryRecord {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        title: "".to_string(),
        minimum: 0.0,
        audience: "".to_string(),
        segment: "".to_string(),
        discount: 0.0,
        maximum: 0.0,
        source: "".to_string(),
        external_ref: "".to_string(),
        status: DefaultQueryRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_query_record("code-540", "label-550");
        assert_eq!(item.status, DefaultQueryRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_default_query_record("code-540", "label-550");
        if item.move_to(DefaultQueryRecordStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultQueryRecordStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_query_record("code-540", "label-550");
        item.status = DefaultQueryRecordStatus::Withdrawn;
        assert!(
            item.move_to(DefaultQueryRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
