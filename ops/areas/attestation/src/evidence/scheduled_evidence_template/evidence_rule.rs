use super::*;

use acme_kit::text;

/// Where a evidence rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EvidenceRuleStatus {
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

impl std::fmt::Display for EvidenceRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a evidence rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRule {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub retries: i64,
    pub revision: i64,
    pub priority: i64,
    pub channel: String,
    pub region: String,
    pub limit: i64,
    pub status: EvidenceRuleStatus,
}

impl EvidenceRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceRuleStatus) -> bool {
        match self.status {
            EvidenceRuleStatus::Draft => {
                return next == EvidenceRuleStatus::Closed;
            }
            EvidenceRuleStatus::Pending => {
                match next {
                    EvidenceRuleStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceRuleStatus::Active => {
                match next {
                    EvidenceRuleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceRuleStatus::Suspended => {
                match next {
                    EvidenceRuleStatus::Closed => {
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

    pub fn move_to(&mut self, next: EvidenceRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.priority as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceRule {
        EvidenceRule {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            retries: self.retries,
            revision: self.revision,
            priority: self.priority,
            channel: self.channel.clone(),
            region: self.region.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a evidence rule in status draft.
pub fn new_evidence_rule(code: &str, label: &str) -> EvidenceRule {
    EvidenceRule {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        retries: 0,
        revision: 0,
        priority: 0,
        channel: "".to_string(),
        region: "".to_string(),
        limit: 0,
        status: EvidenceRuleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_evidence_rule("code-907", "label-146");
        assert_eq!(item.status, EvidenceRuleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_evidence_rule("code-907", "label-146");
        if item.move_to(EvidenceRuleStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceRuleStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_evidence_rule("code-907", "label-146");
        item.status = EvidenceRuleStatus::Closed;
        assert!(
            item.move_to(EvidenceRuleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
