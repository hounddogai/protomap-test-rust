use super::*;

use acme_kit::text;

/// Where a evidence baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceBaselineStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl EvidenceBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for EvidenceBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One evidence baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBaseline {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub archived: bool,
    pub quantity: i64,
    pub region: String,
    pub revision: i64,
    pub ceiling: f64,
    pub weight: f64,
    pub channel: String,
    pub status: EvidenceBaselineStatus,
}

impl EvidenceBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceBaselineStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceBaselineStatus) -> bool {
        match self.status {
            EvidenceBaselineStatus::Requested => {
                return next == EvidenceBaselineStatus::Revoked;
            }
            EvidenceBaselineStatus::Granted => {
                match next {
                    EvidenceBaselineStatus::Revoked | EvidenceBaselineStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceBaselineStatus::Revoked => {
                return next == EvidenceBaselineStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "evidence baseline", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.budget - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceBaseline {
        EvidenceBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            archived: self.archived,
            quantity: self.quantity,
            region: self.region.clone(),
            revision: self.revision,
            ceiling: self.ceiling,
            weight: self.weight,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a evidence baseline in status requested.
pub fn new_evidence_baseline(code: &str, label: &str) -> EvidenceBaseline {
    EvidenceBaseline {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        archived: false,
        quantity: 0,
        region: "".to_string(),
        revision: 0,
        ceiling: 0.0,
        weight: 0.0,
        channel: "".to_string(),
        status: EvidenceBaselineStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_evidence_baseline("code-337", "label-510");
        assert_eq!(item.status, EvidenceBaselineStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_evidence_baseline("code-337", "label-510");
        if item.move_to(EvidenceBaselineStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceBaselineStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_evidence_baseline("code-337", "label-510");
        item.status = EvidenceBaselineStatus::Expired;
        assert!(
            item.move_to(EvidenceBaselineStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
