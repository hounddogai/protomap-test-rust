use super::*;

use acme_kit::text;

/// Where a isolation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationTemplateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IsolationTemplateStatus {
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

impl std::fmt::Display for IsolationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a isolation template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTemplate {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub burst: i64,
    pub fingerprint: String,
    pub grace_days: i64,
    pub checksum: String,
    pub principal: String,
    pub rotation_days: i64,
    pub description: String,
    pub minimum: f64,
    pub status: IsolationTemplateStatus,
}

impl IsolationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationTemplateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationTemplateStatus) -> bool {
        match self.status {
            IsolationTemplateStatus::Draft => {
                match next {
                    IsolationTemplateStatus::Active | IsolationTemplateStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationTemplateStatus::Pending => {
                match next {
                    IsolationTemplateStatus::Suspended | IsolationTemplateStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationTemplateStatus::Active => {
                match next {
                    IsolationTemplateStatus::Suspended | IsolationTemplateStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationTemplateStatus::Suspended => {
                match next {
                    IsolationTemplateStatus::Closed => {
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

    pub fn move_to(&mut self, next: IsolationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "isolation template", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.grace_days as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationTemplate {
        IsolationTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            burst: self.burst,
            fingerprint: self.fingerprint.clone(),
            grace_days: self.grace_days,
            checksum: self.checksum.clone(),
            principal: self.principal.clone(),
            rotation_days: self.rotation_days,
            description: self.description.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a isolation template in status draft.
pub fn new_isolation_template(code: &str, title: &str) -> IsolationTemplate {
    IsolationTemplate {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        burst: 0,
        fingerprint: "".to_string(),
        grace_days: 0,
        checksum: "".to_string(),
        principal: "".to_string(),
        rotation_days: 0,
        description: "".to_string(),
        minimum: 0.0,
        status: IsolationTemplateStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_isolation_template("code-892", "title-843");
        assert_eq!(item.status, IsolationTemplateStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_isolation_template("code-892", "title-843");
        if item.move_to(IsolationTemplateStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationTemplateStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_isolation_template("code-892", "title-843");
        item.status = IsolationTemplateStatus::Closed;
        assert!(
            item.move_to(IsolationTemplateStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
