use super::*;

use acme_kit::text;

/// Where a rolling suspension exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSuspensionExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingSuspensionExceptionStatus {
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

impl std::fmt::Display for RollingSuspensionExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling suspension exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSuspensionException {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub floor_value: f64,
    pub cipher: String,
    pub approved: bool,
    pub amount: f64,
    pub discount: f64,
    pub phone: String,
    pub timezone: String,
    pub description: String,
    pub status: RollingSuspensionExceptionStatus,
}

impl RollingSuspensionException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSuspensionExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSuspensionExceptionStatus) -> bool {
        match self.status {
            RollingSuspensionExceptionStatus::Draft => {
                match next {
                    RollingSuspensionExceptionStatus::Active | RollingSuspensionExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSuspensionExceptionStatus::Pending => {
                match next {
                    RollingSuspensionExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSuspensionExceptionStatus::Active => {
                match next {
                    RollingSuspensionExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSuspensionExceptionStatus::Suspended => {
                match next {
                    RollingSuspensionExceptionStatus::Closed => {
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

    pub fn move_to(&mut self, next: RollingSuspensionExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling suspension exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        (self.floor_value * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingSuspensionException {
        RollingSuspensionException {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            floor_value: self.floor_value,
            cipher: self.cipher.clone(),
            approved: self.approved,
            amount: self.amount,
            discount: self.discount,
            phone: self.phone.clone(),
            timezone: self.timezone.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling suspension exception in status draft.
pub fn new_rolling_suspension_exception(code: &str, name: &str) -> RollingSuspensionException {
    RollingSuspensionException {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        floor_value: 0.0,
        cipher: "".to_string(),
        approved: false,
        amount: 0.0,
        discount: 0.0,
        phone: "".to_string(),
        timezone: "".to_string(),
        description: "".to_string(),
        status: RollingSuspensionExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_suspension_exception("code-870", "name-394");
        assert_eq!(item.status, RollingSuspensionExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_rolling_suspension_exception("code-870", "name-394");
        if item.move_to(RollingSuspensionExceptionStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingSuspensionExceptionStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_suspension_exception("code-870", "name-394");
        item.status = RollingSuspensionExceptionStatus::Closed;
        assert!(
            item.move_to(RollingSuspensionExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
