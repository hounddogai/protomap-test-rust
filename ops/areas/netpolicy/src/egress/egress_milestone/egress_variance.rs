use super::*;

use acme_kit::text;

/// Where a egress variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EgressVarianceStatus {
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

impl std::fmt::Display for EgressVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressVariance {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub burst: i64,
    pub rotation_days: i64,
    pub discount: f64,
    pub notes: String,
    pub status: EgressVarianceStatus,
}

impl EgressVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressVarianceStatus) -> bool {
        match self.status {
            EgressVarianceStatus::Draft => {
                match next {
                    EgressVarianceStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressVarianceStatus::Pending => {
                match next {
                    EgressVarianceStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressVarianceStatus::Active => {
                return next == EgressVarianceStatus::Suspended;
            }
            EgressVarianceStatus::Suspended => {
                match next {
                    EgressVarianceStatus::Closed => {
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

    pub fn move_to(&mut self, next: EgressVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.burst as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressVariance {
        EgressVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            burst: self.burst,
            rotation_days: self.rotation_days,
            discount: self.discount,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a egress variance in status draft.
pub fn new_egress_variance(code: &str, title: &str) -> EgressVariance {
    EgressVariance {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        burst: 0,
        rotation_days: 0,
        discount: 0.0,
        notes: "".to_string(),
        status: EgressVarianceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_egress_variance("code-830", "title-413");
        assert_eq!(item.status, EgressVarianceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_egress_variance("code-830", "title-413");
        if item.move_to(EgressVarianceStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressVarianceStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_egress_variance("code-830", "title-413");
        item.status = EgressVarianceStatus::Closed;
        assert!(
            item.move_to(EgressVarianceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
