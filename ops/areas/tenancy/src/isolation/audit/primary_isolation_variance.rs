use super::*;

use acme_kit::text;

/// Where a primary isolation variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryIsolationVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryIsolationVarianceStatus {
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

impl std::fmt::Display for PrimaryIsolationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary isolation variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIsolationVariance {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub issuer: String,
    pub phone: String,
    pub status: PrimaryIsolationVarianceStatus,
}

impl PrimaryIsolationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryIsolationVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryIsolationVarianceStatus) -> bool {
        match self.status {
            PrimaryIsolationVarianceStatus::Draft => {
                match next {
                    PrimaryIsolationVarianceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryIsolationVarianceStatus::Pending => {
                match next {
                    PrimaryIsolationVarianceStatus::Active | PrimaryIsolationVarianceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryIsolationVarianceStatus::Active => {
                return next == PrimaryIsolationVarianceStatus::Suspended
                    || next == PrimaryIsolationVarianceStatus::Closed;
            }
            PrimaryIsolationVarianceStatus::Suspended => {
                match next {
                    PrimaryIsolationVarianceStatus::Closed => {
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

    pub fn move_to(&mut self, next: PrimaryIsolationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary isolation variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> PrimaryIsolationVariance {
        PrimaryIsolationVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            issuer: self.issuer.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary isolation variance in status draft.
pub fn new_primary_isolation_variance(code: &str, title: &str) -> PrimaryIsolationVariance {
    PrimaryIsolationVariance {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        issuer: "".to_string(),
        phone: "".to_string(),
        status: PrimaryIsolationVarianceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_isolation_variance("code-880", "title-348");
        assert_eq!(item.status, PrimaryIsolationVarianceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_primary_isolation_variance("code-880", "title-348");
        if item.move_to(PrimaryIsolationVarianceStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryIsolationVarianceStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_isolation_variance("code-880", "title-348");
        item.status = PrimaryIsolationVarianceStatus::Closed;
        assert!(
            item.move_to(PrimaryIsolationVarianceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
