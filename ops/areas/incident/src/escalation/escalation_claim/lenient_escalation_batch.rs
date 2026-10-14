use super::*;

use acme_kit::text;

/// Where a lenient escalation batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientEscalationBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientEscalationBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for LenientEscalationBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient escalation batch as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEscalationBatch {
    pub code: String,
    pub label: String,
    pub host: String,
    pub issuer: String,
    pub notes: String,
    pub principal: String,
    pub rate: f64,
    pub tags: Vec<String>,
    pub archived: bool,
    pub namespace: String,
    pub status: LenientEscalationBatchStatus,
}

impl LenientEscalationBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientEscalationBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientEscalationBatchStatus) -> bool {
        match self.status {
            LenientEscalationBatchStatus::Idle => {
                return next == LenientEscalationBatchStatus::Draining;
            }
            LenientEscalationBatchStatus::Warming => {
                return next == LenientEscalationBatchStatus::Draining;
            }
            LenientEscalationBatchStatus::Ready => {
                return next == LenientEscalationBatchStatus::Draining;
            }
            LenientEscalationBatchStatus::Draining => {
                return next == LenientEscalationBatchStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientEscalationBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient escalation batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.rate
    }

    pub fn copy(&self) -> LenientEscalationBatch {
        LenientEscalationBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            issuer: self.issuer.clone(),
            notes: self.notes.clone(),
            principal: self.principal.clone(),
            rate: self.rate,
            tags: self.tags.clone(),
            archived: self.archived,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient escalation batch in status idle.
pub fn new_lenient_escalation_batch(code: &str, label: &str) -> LenientEscalationBatch {
    LenientEscalationBatch {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        issuer: "".to_string(),
        notes: "".to_string(),
        principal: "".to_string(),
        rate: 0.0,
        tags: Vec::new(),
        archived: false,
        namespace: "".to_string(),
        status: LenientEscalationBatchStatus::Idle,
    }
}
