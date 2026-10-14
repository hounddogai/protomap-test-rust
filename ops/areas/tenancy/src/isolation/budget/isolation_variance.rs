use super::*;

use acme_kit::text;

/// Where a isolation variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IsolationVarianceStatus {
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

impl std::fmt::Display for IsolationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationVariance {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub retries: i64,
    pub port: i64,
    pub status: IsolationVarianceStatus,
}

impl IsolationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationVarianceStatus) -> bool {
        match self.status {
            IsolationVarianceStatus::Draft => {
                match next {
                    IsolationVarianceStatus::Pending | IsolationVarianceStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationVarianceStatus::Pending => {
                return next == IsolationVarianceStatus::Active || next == IsolationVarianceStatus::Suspended;
            }
            IsolationVarianceStatus::Active => {
                return next == IsolationVarianceStatus::Closed;
            }
            IsolationVarianceStatus::Suspended => {
                return next == IsolationVarianceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "isolation variance", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.port as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationVariance {
        IsolationVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            retries: self.retries,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a isolation variance in status draft.
pub fn new_isolation_variance(code: &str, title: &str) -> IsolationVariance {
    IsolationVariance {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        retries: 0,
        port: 0,
        status: IsolationVarianceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_isolation_variance("code-113", "title-242");
        assert_eq!(item.status, IsolationVarianceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_isolation_variance("code-113", "title-242");
        if item.move_to(IsolationVarianceStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationVarianceStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_isolation_variance("code-113", "title-242");
        item.status = IsolationVarianceStatus::Closed;
        assert!(
            item.move_to(IsolationVarianceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
