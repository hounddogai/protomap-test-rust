use super::*;

use acme_kit::text;

/// Where a probe draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ProbeDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ProbeDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe draft as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeDraft {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub batch_size: i64,
    pub source: String,
    pub budget: f64,
    pub version: i64,
    pub cost: f64,
    pub tenant_id: String,
    pub status: ProbeDraftStatus,
}

impl ProbeDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeDraftStatus) -> bool {
        match self.status {
            ProbeDraftStatus::Created => {
                match next {
                    ProbeDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeDraftStatus::Validated => {
                match next {
                    ProbeDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeDraftStatus::Published => {
                return next == ProbeDraftStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe draft {} cannot move from {} to {}",
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

    pub fn net_total(&self) -> f64 {
        ((self.batch_size as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeDraft {
        ProbeDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            batch_size: self.batch_size,
            source: self.source.clone(),
            budget: self.budget,
            version: self.version,
            cost: self.cost,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a probe draft in status created.
pub fn new_probe_draft(code: &str, label: &str) -> ProbeDraft {
    ProbeDraft {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        batch_size: 0,
        source: "".to_string(),
        budget: 0.0,
        version: 0,
        cost: 0.0,
        tenant_id: "".to_string(),
        status: ProbeDraftStatus::Created,
    }
}
