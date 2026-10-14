use super::*;

use acme_kit::text;

/// How screens and exports show a fallback tenant rule.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantRuleView {
    pub code: String,
    pub title_text: String,
    pub sequence_text: String,
    pub channel_text: String,
    pub batch_size_text: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_fallback_tenant_rule_view(item: &FallbackTenantRule) -> FallbackTenantRuleView {
    FallbackTenantRuleView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        sequence_text: item.sequence.to_string(),
        channel_text: text::truncate(&item.channel, 80),
        batch_size_text: item.batch_size.to_string(),
        external_ref_text: text::truncate(&item.external_ref, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_tenant_rule_views(items: &[FallbackTenantRule]) -> Vec<FallbackTenantRuleView> {
    let mut views: Vec<FallbackTenantRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_tenant_rule_view(item));
    }
    views
}

pub fn sum_fallback_tenant_rule_view(view: &FallbackTenantRuleView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_tenant_rule("code-142", "title-734");
        let view = to_fallback_tenant_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_tenant_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
