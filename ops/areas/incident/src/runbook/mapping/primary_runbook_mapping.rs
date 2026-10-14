use super::*;

use acme_kit::text;

/// Where a primary runbook mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRunbookMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryRunbookMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryRunbookMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary runbook mapping as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookMapping {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub summary: String,
    pub sequence: i64,
    pub locale: String,
    pub visible: bool,
    pub segment: String,
    pub ceiling: f64,
    pub floor_value: f64,
    pub cost: f64,
    pub status: PrimaryRunbookMappingStatus,
}

impl PrimaryRunbookMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRunbookMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRunbookMappingStatus) -> bool {
        match self.status {
            PrimaryRunbookMappingStatus::Proposed => {
                return next == PrimaryRunbookMappingStatus::Rejected || next == PrimaryRunbookMappingStatus::Withdrawn;
            }
            PrimaryRunbookMappingStatus::Approved => {
                return next == PrimaryRunbookMappingStatus::Rejected;
            }
            PrimaryRunbookMappingStatus::Rejected => {
                match next {
                    PrimaryRunbookMappingStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PrimaryRunbookMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary runbook mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        (self.floor_value * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRunbookMapping {
        PrimaryRunbookMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            summary: self.summary.clone(),
            sequence: self.sequence,
            locale: self.locale.clone(),
            visible: self.visible,
            segment: self.segment.clone(),
            ceiling: self.ceiling,
            floor_value: self.floor_value,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a primary runbook mapping in status proposed.
pub fn new_primary_runbook_mapping(code: &str, label: &str) -> PrimaryRunbookMapping {
    PrimaryRunbookMapping {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        summary: "".to_string(),
        sequence: 0,
        locale: "".to_string(),
        visible: false,
        segment: "".to_string(),
        ceiling: 0.0,
        floor_value: 0.0,
        cost: 0.0,
        status: PrimaryRunbookMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_runbook_mapping("code-433", "label-805");
        assert_eq!(item.status, PrimaryRunbookMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_primary_runbook_mapping("code-433", "label-805");
        if item.move_to(PrimaryRunbookMappingStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRunbookMappingStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_runbook_mapping("code-433", "label-805");
        item.status = PrimaryRunbookMappingStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryRunbookMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
