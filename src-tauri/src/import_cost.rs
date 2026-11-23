//! Transparent prototype landed-cost allocation.
//!
//! This is an internal management-cost worksheet only. It does not calculate
//! customs value, customs duty, tax liability, inventory valuation under an
//! accounting framework, or a recommended selling price. Those treatments
//! remain subject to applicable law and approved practitioner policy.

use serde::{Deserialize, Serialize};

const QUANTITY_SCALE: i128 = 1_000_000;

#[derive(Debug, Clone, Deserialize)]
pub struct GoodsLineInput {
    pub item_reference: String,
    pub description: String,
    /// Quantity expressed in millionths (e.g. 1.25 units = 1_250_000).
    pub quantity_micros: i64,
    /// Unit price in source-currency minor units.
    pub unit_price_minor: i64,
    pub currency_code: String,
    pub source_reference: String,
    pub fx_rate_source: String,
    pub fx_rate_date: String,
    pub fx_authorized_by: String,
    /// Base-currency minor units per source-currency minor unit, as a ratio.
    pub fx_numerator: i64,
    pub fx_denominator: i64,
    /// Optional allocation weights expressed as positive integer millionths.
    pub weight_micros: i64,
    pub volume_micros: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CostComponentInput {
    pub component_reference: String,
    pub category: String,
    /// Amount in source-currency minor units.
    pub amount_minor: i64,
    pub currency_code: String,
    pub source_reference: String,
    pub fx_rate_source: String,
    pub fx_rate_date: String,
    pub fx_authorized_by: String,
    pub fx_numerator: i64,
    pub fx_denominator: i64,
    /// Direct, Quantity, Weight, Volume, or Value.
    pub allocation_basis: String,
    /// Required only when allocation_basis is Direct.
    pub direct_item_reference: String,
    /// Capitalize, Exclude, Recoverable tax, Customs-value-only, Needs review.
    pub treatment: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LineResult {
    pub item_reference: String,
    pub description: String,
    pub source_reference: String,
    pub fx_rate_source: String,
    pub fx_rate_date: String,
    pub fx_authorized_by: String,
    pub quantity_micros: i64,
    pub goods_value_minor: i64,
    pub allocated_cost_minor: i64,
    pub landed_cost_minor: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentResult {
    pub component_reference: String,
    pub category: String,
    pub source_reference: String,
    pub fx_rate_source: String,
    pub fx_rate_date: String,
    pub fx_authorized_by: String,
    pub source_currency: String,
    pub source_amount_minor: i64,
    pub base_amount_minor: i64,
    pub treatment: String,
    pub allocation_basis: String,
    pub allocations: Vec<ItemAllocation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemAllocation {
    pub item_reference: String,
    pub amount_minor: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportCostResult {
    pub base_currency: String,
    pub total_goods_value_minor: i64,
    pub total_capitalized_cost_minor: i64,
    pub total_landed_cost_minor: i64,
    pub excluded_or_pending_minor: i64,
    pub lines: Vec<LineResult>,
    pub components: Vec<ComponentResult>,
    pub limitations: Vec<String>,
}

fn checked_i64(value: i128, label: &str) -> Result<i64, String> {
    i64::try_from(value).map_err(|_| format!("{label} exceeds the supported prototype range."))
}

fn rounded_ratio(numerator: i128, denominator: i128, label: &str) -> Result<i128, String> {
    if denominator <= 0 || numerator < 0 {
        return Err(format!("{label} requires non-negative amounts and a positive denominator."));
    }
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let rounded = if remainder.saturating_mul(2) >= denominator {
        quotient.checked_add(1).ok_or_else(|| format!("{label} overflow."))?
    } else {
        quotient
    };
    Ok(rounded)
}

fn convert_to_base(amount_minor: i64, numerator: i64, denominator: i64, label: &str) -> Result<i64, String> {
    if amount_minor < 0 || numerator <= 0 || denominator <= 0 {
        return Err(format!("{label}: amount must be non-negative and FX numerator/denominator must be positive."));
    }
    let product = i128::from(amount_minor)
        .checked_mul(i128::from(numerator))
        .ok_or_else(|| format!("{label}: FX conversion overflow."))?;
    checked_i64(rounded_ratio(product, i128::from(denominator), label)?, label)
}

fn allocate_largest_remainder(total: i64, weights: &[i128]) -> Result<Vec<i64>, String> {
    if total < 0 || weights.is_empty() || weights.iter().any(|weight| *weight <= 0) {
        return Err("Allocation requires a non-negative amount and positive weights for every item.".into());
    }
    let weight_sum = weights.iter().try_fold(0_i128, |sum, weight| sum.checked_add(*weight))
        .ok_or_else(|| "Allocation weight total overflow.".to_string())?;
    if weight_sum <= 0 {
        return Err("Allocation weight total must be positive.".into());
    }
    let mut allocations = Vec::with_capacity(weights.len());
    let mut remainders = Vec::with_capacity(weights.len());
    let mut allocated = 0_i128;
    for (index, weight) in weights.iter().enumerate() {
        let product = i128::from(total).checked_mul(*weight)
            .ok_or_else(|| "Allocation multiplication overflow.".to_string())?;
        let share = product / weight_sum;
        let remainder = product % weight_sum;
        allocated = allocated.checked_add(share)
            .ok_or_else(|| "Allocation total overflow.".to_string())?;
        allocations.push(share);
        remainders.push((index, remainder));
    }
    let remaining = i128::from(total) - allocated;
    remainders.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    for (index, _) in remainders.into_iter().take(usize::try_from(remaining).unwrap_or(usize::MAX)) {
        allocations[index] = allocations[index].checked_add(1)
            .ok_or_else(|| "Allocation remainder overflow.".to_string())?;
    }
    allocations.into_iter().map(|value| checked_i64(value, "Allocation")).collect()
}

pub fn calculate_import_cost(
    base_currency: &str,
    goods: &[GoodsLineInput],
    components: &[CostComponentInput],
) -> Result<ImportCostResult, String> {
    let base_currency = base_currency.trim().to_ascii_uppercase();
    if base_currency.len() != 3 || !base_currency.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err("Select a three-letter base currency code.".into());
    }
    if goods.is_empty() {
        return Err("Add at least one goods line before calculating an import-cost preview.".into());
    }

    let mut seen = std::collections::HashSet::new();
    let mut lines = Vec::with_capacity(goods.len());
    for line in goods {
        let reference = line.item_reference.trim();
        if reference.is_empty() || !seen.insert(reference.to_string()) {
            return Err("Every goods line needs a unique, non-empty item reference.".into());
        }
        if line.quantity_micros <= 0 || line.unit_price_minor < 0 {
            return Err(format!("Goods line {reference} needs a positive quantity and non-negative unit price."));
        }
        if line.source_reference.trim().is_empty() || line.fx_rate_source.trim().is_empty() || line.fx_rate_date.trim().is_empty() || line.fx_authorized_by.trim().is_empty() {
            return Err(format!("Goods line {reference} needs source evidence and an explicit FX source/date/authorization (use a documented 1:1 basis for the base currency)."));
        }
        if line.currency_code.trim().len() != 3 || !line.currency_code.trim().chars().all(|ch| ch.is_ascii_alphabetic()) {
            return Err(format!("Goods line {reference} needs a three-letter alphabetic source currency."));
        }
        if line.weight_micros < 0 || line.volume_micros < 0 {
            return Err(format!("Goods line {reference} cannot have negative weight or volume."));
        }
        let extended = i128::from(line.unit_price_minor)
            .checked_mul(i128::from(line.quantity_micros))
            .ok_or_else(|| format!("Goods line {reference} value overflow."))?;
        let source_total = checked_i64(rounded_ratio(extended, QUANTITY_SCALE, "Goods line quantity")?, "Goods line value")?;
        let goods_value = convert_to_base(source_total, line.fx_numerator, line.fx_denominator, "Goods line FX")?;
        lines.push(LineResult {
            item_reference: reference.to_string(),
            description: line.description.trim().to_string(),
            source_reference: line.source_reference.trim().to_string(),
            fx_rate_source: line.fx_rate_source.trim().to_string(),
            fx_rate_date: line.fx_rate_date.trim().to_string(),
            fx_authorized_by: line.fx_authorized_by.trim().to_string(),
            quantity_micros: line.quantity_micros,
            goods_value_minor: goods_value,
            allocated_cost_minor: 0,
            landed_cost_minor: goods_value,
        });
    }

    let mut component_results = Vec::with_capacity(components.len());
    let mut capitalized_total = 0_i64;
    let mut excluded_total = 0_i64;

    for component in components {
        let reference = component.component_reference.trim();
        if reference.is_empty() || component.category.trim().is_empty() {
            return Err("Every cost component needs a reference and category.".into());
        }
        if component.source_reference.trim().is_empty() || component.fx_rate_source.trim().is_empty() || component.fx_rate_date.trim().is_empty() || component.fx_authorized_by.trim().is_empty() {
            return Err(format!("Cost component {reference} needs source evidence and an explicit FX source/date/authorization (use a documented 1:1 basis for the base currency)."));
        }
        if component.currency_code.trim().len() != 3 || !component.currency_code.trim().chars().all(|ch| ch.is_ascii_alphabetic()) {
            return Err(format!("Cost component {reference} needs a three-letter alphabetic source currency."));
        }
        let base_amount = convert_to_base(component.amount_minor, component.fx_numerator, component.fx_denominator, "Cost component FX")?;
        let treatment = component.treatment.trim();
        let basis = component.allocation_basis.trim();
        let mut allocations = Vec::new();

        match treatment {
            "Capitalize" => {
                let shares = if basis == "Direct" {
                    let target = component.direct_item_reference.trim();
                    if target.is_empty() {
                        return Err(format!("Direct component {reference} needs an item reference."));
                    }
                    let index = lines.iter().position(|line| line.item_reference == target)
                        .ok_or_else(|| format!("Direct component {reference} points to an unknown item {target}."))?;
                    let mut values = vec![0_i64; lines.len()];
                    values[index] = base_amount;
                    values
                } else {
                    let weights: Vec<i128> = lines.iter().map(|line| match basis {
                        "Quantity" => Ok(i128::from(line.quantity_micros)),
                        "Weight" => if line.weight_micros > 0 { Ok(i128::from(line.weight_micros)) } else { Err(format!("Weight allocation requires a positive weight for item {}.", line.item_reference)) },
                        "Volume" => if line.volume_micros > 0 { Ok(i128::from(line.volume_micros)) } else { Err(format!("Volume allocation requires a positive volume for item {}.", line.item_reference)) },
                        "Value" => if line.goods_value_minor > 0 { Ok(i128::from(line.goods_value_minor)) } else { Err(format!("Value allocation requires a positive goods value for item {}.", line.item_reference)) },
                        _ => Err(format!("Unsupported allocation basis {basis}. Choose Direct, Quantity, Weight, Volume, or Value.")),
                    }).collect::<Result<Vec<_>,_>>()?;
                    allocate_largest_remainder(base_amount, &weights)?
                };
                for (index, amount) in shares.iter().enumerate() {
                    if *amount > 0 {
                        lines[index].allocated_cost_minor = lines[index].allocated_cost_minor.checked_add(*amount)
                            .ok_or_else(|| "Item allocated cost overflow.".to_string())?;
                        lines[index].landed_cost_minor = lines[index].landed_cost_minor.checked_add(*amount)
                            .ok_or_else(|| "Item landed cost overflow.".to_string())?;
                        allocations.push(ItemAllocation { item_reference: lines[index].item_reference.clone(), amount_minor: *amount });
                    }
                }
                capitalized_total = capitalized_total.checked_add(base_amount)
                    .ok_or_else(|| "Capitalized cost total overflow.".to_string())?;
            }
            "Exclude" | "Recoverable tax" | "Customs-value-only" | "Needs review" => {
                excluded_total = excluded_total.checked_add(base_amount)
                    .ok_or_else(|| "Excluded/pending cost total overflow.".to_string())?;
            }
            _ => return Err(format!("Unsupported treatment {treatment}. Choose Capitalize, Exclude, Recoverable tax, Customs-value-only, or Needs review.")),
        }

        component_results.push(ComponentResult {
            component_reference: reference.to_string(),
            category: component.category.trim().to_string(),
            source_reference: component.source_reference.trim().to_string(),
            fx_rate_source: component.fx_rate_source.trim().to_string(),
            fx_rate_date: component.fx_rate_date.trim().to_string(),
            fx_authorized_by: component.fx_authorized_by.trim().to_string(),
            source_currency: component.currency_code.trim().to_ascii_uppercase(),
            source_amount_minor: component.amount_minor,
            base_amount_minor: base_amount,
            treatment: treatment.to_string(),
            allocation_basis: basis.to_string(),
            allocations,
        });
    }

    let goods_total = lines.iter().try_fold(0_i64, |sum, line| sum.checked_add(line.goods_value_minor))
        .ok_or_else(|| "Goods value total overflow.".to_string())?;
    let landed_total = lines.iter().try_fold(0_i64, |sum, line| sum.checked_add(line.landed_cost_minor))
        .ok_or_else(|| "Landed cost total overflow.".to_string())?;

    Ok(ImportCostResult {
        base_currency,
        total_goods_value_minor: goods_total,
        total_capitalized_cost_minor: capitalized_total,
        total_landed_cost_minor: landed_total,
        excluded_or_pending_minor: excluded_total,
        lines,
        components: component_results,
        limitations: vec![
            "Draft management-cost allocation only; not a customs valuation or customs declaration.".into(),
            "Incoterms identify contractual cost/risk responsibilities but do not determine customs value or accounting treatment.".into(),
            "Cost capitalization, recoverability, FX source/date and statutory treatment require the applicable approved policy.".into(),
            "All amounts are integer minor units; currency precision must be configured and independently reviewed before real use.".into(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn goods(reference: &str, value: i64, quantity: i64, weight: i64) -> GoodsLineInput {
        GoodsLineInput {
            item_reference: reference.into(), description: reference.into(),
            quantity_micros: quantity, unit_price_minor: value, currency_code: "ETB".into(),
            source_reference: "synthetic-source".into(), fx_rate_source: "base-currency 1:1".into(), fx_rate_date: "2026-01-01".into(), fx_authorized_by: "synthetic fixture".into(),
            fx_numerator: 1, fx_denominator: 1, weight_micros: weight, volume_micros: weight,
        }
    }

    #[test]
    fn shared_cost_allocation_preserves_every_minor_unit() {
        let lines = vec![goods("A", 1000, 1_000_000, 1_000_000), goods("B", 1000, 1_000_000, 2_000_000), goods("C", 1000, 1_000_000, 3_000_000)];
        let components = vec![CostComponentInput {
            component_reference: "FREIGHT-1".into(), category: "Freight".into(), amount_minor: 101,
            currency_code: "ETB".into(), source_reference: "synthetic-source".into(), fx_rate_source: "base-currency 1:1".into(), fx_rate_date: "2026-01-01".into(), fx_authorized_by: "synthetic fixture".into(), fx_numerator: 1, fx_denominator: 1,
            allocation_basis: "Weight".into(), direct_item_reference: "".into(), treatment: "Capitalize".into(),
        }];
        let result = calculate_import_cost("ETB", &lines, &components).unwrap();
        assert_eq!(result.total_capitalized_cost_minor, 101);
        assert_eq!(result.lines.iter().map(|line| line.allocated_cost_minor).sum::<i64>(), 101);
        assert_eq!(result.total_landed_cost_minor, 3101);
    }

    #[test]
    fn recoverable_tax_is_visible_but_not_capitalized() {
        let lines = vec![goods("A", 1000, 1_000_000, 1_000_000)];
        let components = vec![CostComponentInput {
            component_reference: "VAT-1".into(), category: "VAT".into(), amount_minor: 150,
            currency_code: "ETB".into(), source_reference: "synthetic-source".into(), fx_rate_source: "base-currency 1:1".into(), fx_rate_date: "2026-01-01".into(), fx_authorized_by: "synthetic fixture".into(), fx_numerator: 1, fx_denominator: 1,
            allocation_basis: "Direct".into(), direct_item_reference: "A".into(), treatment: "Recoverable tax".into(),
        }];
        let result = calculate_import_cost("ETB", &lines, &components).unwrap();
        assert_eq!(result.total_landed_cost_minor, 1000);
        assert_eq!(result.excluded_or_pending_minor, 150);
        assert!(result.components[0].allocations.is_empty());
    }

    #[test]
    fn direct_component_is_allocated_to_one_named_item() {
        let lines = vec![goods("A", 1000, 1_000_000, 1), goods("B", 2000, 1_000_000, 1)];
        let components = vec![CostComponentInput {
            component_reference: "HANDLING-A".into(), category: "Handling".into(), amount_minor: 25,
            currency_code: "ETB".into(), source_reference: "synthetic-source".into(), fx_rate_source: "base-currency 1:1".into(), fx_rate_date: "2026-01-01".into(), fx_authorized_by: "synthetic fixture".into(), fx_numerator: 1, fx_denominator: 1,
            allocation_basis: "Direct".into(), direct_item_reference: "A".into(), treatment: "Capitalize".into(),
        }];
        let result = calculate_import_cost("ETB", &lines, &components).unwrap();
        assert_eq!(result.lines[0].landed_cost_minor, 1025);
        assert_eq!(result.lines[1].landed_cost_minor, 2000);
    }

    #[test]
    fn rejects_missing_weight_and_invalid_fx_instead_of_guessing() {
        let lines = vec![goods("A", 1000, 1_000_000, 0)];
        let components = vec![CostComponentInput {
            component_reference: "FREIGHT".into(), category: "Freight".into(), amount_minor: 10,
            currency_code: "USD".into(), source_reference: "synthetic-source".into(), fx_rate_source: "synthetic test".into(), fx_rate_date: "2026-01-01".into(), fx_authorized_by: "synthetic fixture".into(), fx_numerator: 0, fx_denominator: 1,
            allocation_basis: "Weight".into(), direct_item_reference: "".into(), treatment: "Capitalize".into(),
        }];
        assert!(calculate_import_cost("ETB", &lines, &components).is_err());
    }

    #[test]
    fn customs_value_only_and_pending_items_are_not_mixed_into_landed_cost() {
        let lines = vec![goods("A", 1000, 1_000_000, 1)];
        let components = vec![
            CostComponentInput { component_reference:"CV".into(),category:"Customs base reference".into(),amount_minor:800,currency_code:"ETB".into(),source_reference:"synthetic-source".into(),fx_rate_source:"base-currency 1:1".into(),fx_rate_date:"2026-01-01".into(),fx_authorized_by:"synthetic fixture".into(),fx_numerator:1,fx_denominator:1,allocation_basis:"Direct".into(),direct_item_reference:"A".into(),treatment:"Customs-value-only".into() },
            CostComponentInput { component_reference:"PENDING".into(),category:"Unknown levy".into(),amount_minor:50,currency_code:"ETB".into(),source_reference:"synthetic-source".into(),fx_rate_source:"base-currency 1:1".into(),fx_rate_date:"2026-01-01".into(),fx_authorized_by:"synthetic fixture".into(),fx_numerator:1,fx_denominator:1,allocation_basis:"Direct".into(),direct_item_reference:"A".into(),treatment:"Needs review".into() },
        ];
        let result = calculate_import_cost("ETB", &lines, &components).unwrap();
        assert_eq!(result.total_landed_cost_minor, 1000);
        assert_eq!(result.excluded_or_pending_minor, 850);
    }
}
