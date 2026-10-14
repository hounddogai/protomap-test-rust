use super::*;

use acme_kit::text;

/// Where a probe policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbePolicyStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ProbePolicyStatus {
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

impl std::fmt::Display for ProbePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a probe policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbePolicy {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub channel: String,
    pub contact_email: String,
    pub segment: String,
    pub priority: i64,
    pub rate: f64,
    pub minimum: f64,
    pub title: String,
    pub status: ProbePolicyStatus,
}

impl ProbePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbePolicyStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbePolicyStatus) -> bool {
        match self.status {
            ProbePolicyStatus::Draft => {
                match next {
                    ProbePolicyStatus::Active | ProbePolicyStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbePolicyStatus::Pending => {
                match next {
                    ProbePolicyStatus::Active | ProbePolicyStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbePolicyStatus::Active => {
                return next == ProbePolicyStatus::Suspended;
            }
            ProbePolicyStatus::Suspended => {
                return next == ProbePolicyStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.minimum + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbePolicy {
        ProbePolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            channel: self.channel.clone(),
            contact_email: self.contact_email.clone(),
            segment: self.segment.clone(),
            priority: self.priority,
            rate: self.rate,
            minimum: self.minimum,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a probe policy in status draft.
pub fn new_probe_policy(code: &str, label: &str) -> ProbePolicy {
    ProbePolicy {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        channel: "".to_string(),
        contact_email: "".to_string(),
        segment: "".to_string(),
        priority: 0,
        rate: 0.0,
        minimum: 0.0,
        title: "".to_string(),
        status: ProbePolicyStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_probe_policy("code-462", "label-670");
        assert_eq!(item.status, ProbePolicyStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_probe_policy("code-462", "label-670");
        if item.move_to(ProbePolicyStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbePolicyStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_probe_policy("code-462", "label-670");
        item.status = ProbePolicyStatus::Closed;
        assert!(item.move_to(ProbePolicyStatus::Draft).is_err(), "expected move to fail");
    }
}
