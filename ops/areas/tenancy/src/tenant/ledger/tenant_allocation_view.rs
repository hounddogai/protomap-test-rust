use super::*;

/// How screens and exports show a tenant allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantAllocationView {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub issuer_text: String,
    pub created_at: String,
    pub scope_text: String,
    pub timezone: String,
    pub batch_size: String,
    pub pinned_text: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_tenant_allocation_view(item: &TenantAllocation) -> TenantAllocationView {
    TenantAllocationView {
        code: item.code.clone(),
        label: item.label.clone(),
        principal: item.principal.clone(),
        issuer_text: item.issuer.clone(),
        created_at: item.created_at.to_string(),
        scope_text: item.scope.clone(),
        timezone: item.timezone.clone(),
        batch_size: item.batch_size.to_string(),
        pinned_text: format!("{}", item.pinned),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_allocation_views(items: &[TenantAllocation]) -> Vec<TenantAllocationView> {
    let mut views: Vec<TenantAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_allocation_view(item));
    }
    views
}

pub fn sum_tenant_allocation_view(view: &TenantAllocationView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_batch_size as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_allocation("code-247", "label-210");
        let view = to_tenant_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
