use super::*;

use acme_kit::text;

/// Where a canary summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanarySummaryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CanarySummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CanarySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanarySummary {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub cluster: String,
    pub external_ref: String,
    pub threshold: f64,
    pub key_id: String,
    pub maximum: f64,
    pub unit: String,
    pub contact_email: String,
    pub host: String,
    pub status: CanarySummaryStatus,
}

impl CanarySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanarySummaryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanarySummaryStatus) -> bool {
        match self.status {
            CanarySummaryStatus::Proposed => {
                match next {
                    CanarySummaryStatus::Rejected | CanarySummaryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanarySummaryStatus::Approved => {
                match next {
                    CanarySummaryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanarySummaryStatus::Rejected => {
                match next {
                    CanarySummaryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: CanarySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.threshold - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanarySummary {
        CanarySummary {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            cluster: self.cluster.clone(),
            external_ref: self.external_ref.clone(),
            threshold: self.threshold,
            key_id: self.key_id.clone(),
            maximum: self.maximum,
            unit: self.unit.clone(),
            contact_email: self.contact_email.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a canary summary in status proposed.
pub fn new_canary_summary(code: &str, name: &str) -> CanarySummary {
    CanarySummary {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        cluster: "".to_string(),
        external_ref: "".to_string(),
        threshold: 0.0,
        key_id: "".to_string(),
        maximum: 0.0,
        unit: "".to_string(),
        contact_email: "".to_string(),
        host: "".to_string(),
        status: CanarySummaryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_canary_summary("code-317", "name-33");
        assert_eq!(item.status, CanarySummaryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_canary_summary("code-317", "name-33");
        if item.move_to(CanarySummaryStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanarySummaryStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_canary_summary("code-317", "name-33");
        item.status = CanarySummaryStatus::Withdrawn;
        assert!(
            item.move_to(CanarySummaryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
