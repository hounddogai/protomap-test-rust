use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a evidence group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EvidenceGroupStatus {
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

impl std::fmt::Display for EvidenceGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a evidence group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceGroup {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub active: bool,
    pub attributes: BTreeMap<String, String>,
    pub status: EvidenceGroupStatus,
}

impl EvidenceGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceGroupStatus) -> bool {
        match self.status {
            EvidenceGroupStatus::Idle => {
                return next == EvidenceGroupStatus::Ready;
            }
            EvidenceGroupStatus::Warming => {
                match next {
                    EvidenceGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceGroupStatus::Ready => {
                match next {
                    EvidenceGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceGroupStatus::Draining => {
                return next == EvidenceGroupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> EvidenceGroup {
        EvidenceGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            active: self.active,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a evidence group in status idle.
pub fn new_evidence_group(code: &str, label: &str) -> EvidenceGroup {
    EvidenceGroup {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        active: false,
        attributes: BTreeMap::new(),
        status: EvidenceGroupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_evidence_group("code-818", "label-189");
        assert_eq!(item.status, EvidenceGroupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_evidence_group("code-818", "label-189");
        if item.move_to(EvidenceGroupStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceGroupStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_evidence_group("code-818", "label-189");
        item.status = EvidenceGroupStatus::Stopped;
        assert!(
            item.move_to(EvidenceGroupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
