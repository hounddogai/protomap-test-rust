use super::*;

use acme_kit::text;

/// Where a escalation variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EscalationVarianceStatus {
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

impl std::fmt::Display for EscalationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a escalation variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationVariance {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub starts_at: i64,
    pub grace_days: i64,
    pub capacity: i64,
    pub region: String,
    pub status: EscalationVarianceStatus,
}

impl EscalationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationVarianceStatus) -> bool {
        match self.status {
            EscalationVarianceStatus::Draft => {
                match next {
                    EscalationVarianceStatus::Active | EscalationVarianceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationVarianceStatus::Pending => {
                return next == EscalationVarianceStatus::Active || next == EscalationVarianceStatus::Suspended;
            }
            EscalationVarianceStatus::Active => {
                match next {
                    EscalationVarianceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationVarianceStatus::Suspended => {
                match next {
                    EscalationVarianceStatus::Closed => {
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

    pub fn move_to(&mut self, next: EscalationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.grace_days as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationVariance {
        EscalationVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            starts_at: self.starts_at,
            grace_days: self.grace_days,
            capacity: self.capacity,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a escalation variance in status draft.
pub fn new_escalation_variance(code: &str, label: &str) -> EscalationVariance {
    EscalationVariance {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        starts_at: 0,
        grace_days: 0,
        capacity: 0,
        region: "".to_string(),
        status: EscalationVarianceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_escalation_variance("code-631", "label-196");
        assert_eq!(item.status, EscalationVarianceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_escalation_variance("code-631", "label-196");
        if item.move_to(EscalationVarianceStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationVarianceStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_escalation_variance("code-631", "label-196");
        item.status = EscalationVarianceStatus::Closed;
        assert!(
            item.move_to(EscalationVarianceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
