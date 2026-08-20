//! Payroll calculation engine for DRC (Democratic Republic of Congo) compliance.
//!
//! Implements:
//! - IPR (Impôt Professionnel sur les Revenus) - progressive income tax
//! - CNSS (Caisse Nationale de Sécurité Sociale) - social security contributions
//! - SMIG (Salaire Minimum Interprofessionnel Garanti) - minimum wage enforcement

use nexora_common::money::{
    CurrencyCode, Money, MoneyError, MoneyResult, RoundingPolicy, DEFAULT_SCALE,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// DRC 2025 IPR (Income Tax) brackets.
/// Rates are applied progressively on taxable income.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IprBracket {
    pub from: Decimal,
    pub to: Option<Decimal>, // None = no upper limit (top bracket)
    pub rate: Decimal,       // e.g., 0.15 for 15%
}

impl IprBracket {
    pub fn contains(&self, amount: Decimal) -> bool {
        amount >= self.from && self.to.map_or(true, |t| amount <= t)
    }
}

/// CNSS configuration for a given period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CnssConfig {
    pub employee_rate: Decimal, // 5% (0.05)
    pub employer_rate: Decimal, // 10.5% (0.105)
    pub ceiling: Decimal,       // Monthly ceiling in CDF (1,500,000 CDF for 2025)
}

/// SMIG (Minimum Wage) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmigConfig {
    pub daily: Decimal,      // Daily rate (7,000 CDF for 2025)
    pub monthly_26: Decimal, // 26-day month (182,000 CDF)
    pub monthly_30: Decimal, // 30-day month (210,000 CDF)
}

/// Complete payroll configuration for a country/period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollConfig {
    pub config_version: String, // e.g., "ipr-2025"
    pub ipr_brackets: Vec<IprBracket>,
    pub cnss: CnssConfig,
    pub smig: SmigConfig,
    pub currency: CurrencyCode, // Should be CDF for DRC
}

impl Default for PayrollConfig {
    fn default() -> Self {
        // DRC 2025 defaults (illustrative; actual values require legal review)
        Self {
            config_version: "ipr-2025".to_string(),
            ipr_brackets: vec![
                IprBracket {
                    from: Decimal::ZERO,
                    to: Some(Decimal::new(180000, 0)),
                    rate: Decimal::ZERO,
                },
                IprBracket {
                    from: Decimal::new(180001, 0),
                    to: Some(Decimal::new(480000, 0)),
                    rate: Decimal::new(15, 2),
                }, // 15%
                IprBracket {
                    from: Decimal::new(480001, 0),
                    to: Some(Decimal::new(1200000, 0)),
                    rate: Decimal::new(25, 2),
                }, // 25%
                IprBracket {
                    from: Decimal::new(1200001, 0),
                    to: Some(Decimal::new(2400000, 0)),
                    rate: Decimal::new(30, 2),
                }, // 30%
                IprBracket {
                    from: Decimal::new(2400001, 0),
                    to: Some(Decimal::new(4800000, 0)),
                    rate: Decimal::new(35, 2),
                }, // 35%
                IprBracket {
                    from: Decimal::new(4800001, 0),
                    to: None,
                    rate: Decimal::new(40, 2),
                }, // 40%
            ],
            cnss: CnssConfig {
                employee_rate: Decimal::new(5, 2),   // 5%
                employer_rate: Decimal::new(105, 3), // 10.5%
                ceiling: Decimal::new(1500000, 0),   // 1,500,000 CDF
            },
            smig: SmigConfig {
                daily: Decimal::new(7000, 0),
                monthly_26: Decimal::new(182000, 0),
                monthly_30: Decimal::new(210000, 0),
            },
            currency: CurrencyCode::cdf(),
        }
    }
}

/// Input for calculating a single employee's payroll.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollInput {
    pub employee_id: uuid::Uuid,
    pub gross_pay: Money,                  // Base gross pay for the period
    pub overtime_hours: Option<Decimal>,   // Overtime hours
    pub overtime_rate: Option<Decimal>,    // Overtime multiplier (e.g., 1.5)
    pub allowances: Vec<PayrollAllowance>, // Taxable allowances
    pub deductions: Vec<PayrollDeduction>, // Pre-tax deductions
    pub days_worked: u32,                  // Days worked in period (for SMIG check)
}

/// Taxable allowance (added to gross for tax calculation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollAllowance {
    pub code: String, // e.g., "HOUSING", "TRANSPORT"
    pub description: String,
    pub amount: Money,
    pub is_taxable: bool,
    pub is_cnssable: bool,
}

/// Pre-tax deduction (reduces taxable income).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollDeduction {
    pub code: String,
    pub description: String,
    pub amount: Money,
}

/// Result of payroll calculation for one employee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollResult {
    pub employee_id: uuid::Uuid,
    pub gross_pay: Money,        // Base gross
    pub overtime_pay: Money,     // Calculated overtime
    pub total_allowances: Money, // Sum of allowances
    pub taxable_gross: Money,    // Gross + taxable allowances - pre-tax deductions
    pub ipr_deduction: Money,    // IPR tax
    pub cnss_employee: Money,    // Employee CNSS
    pub cnss_employer: Money,    // Employer CNSS (not deducted from employee)
    pub total_deductions: Money, // IPR + CNSS employee + other deductions
    pub net_pay: Money,          // Taxable gross - total_deductions
    pub employer_cost: Money,    // Gross + employer CNSS + employer contributions
    pub smig_compliant: bool,    // Whether net_pay >= SMIG for days worked
    pub calculation_details: CalculationDetails,
}

/// Detailed breakdown for audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationDetails {
    pub ipr_brackets_applied: Vec<IprBracketApplication>,
    pub cnss_base: Money,      // Base used for CNSS (capped at ceiling)
    pub smig_threshold: Money, // SMIG for days worked
    pub rounding_policy: RoundingPolicy,
}

/// IPR bracket application detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IprBracketApplication {
    pub bracket_from: Decimal,
    pub bracket_to: Option<Decimal>,
    pub rate: Decimal,
    pub taxable_in_bracket: Decimal,
    pub tax_in_bracket: Decimal,
}

/// Calculate IPR (progressive income tax) for DRC.
pub fn calculate_ipr(
    taxable_income: Money,
    config: &PayrollConfig,
) -> MoneyResult<(Money, Vec<IprBracketApplication>)> {
    if taxable_income.currency != config.currency {
        return Err(MoneyError::CurrencyMismatch(
            taxable_income.currency.to_string(),
            config.currency.to_string(),
        ));
    }

    let mut remaining = taxable_income.amount;
    let mut total_tax = Decimal::ZERO;
    let mut applications = Vec::new();
    // Ceiling of the previous band. Bands are declared with a 1-unit gap
    // (…180000 / 180001…), so each band's true width is `to - prev_ceiling`.
    let mut band_floor = Decimal::ZERO;

    for bracket in &config.ipr_brackets {
        if remaining <= Decimal::ZERO {
            break;
        }

        // The top bracket has no upper limit: everything still untaxed falls in
        // it. Computing `Decimal::MAX - from` would overflow, so branch instead.
        //
        // Bands are declared with a 1-unit gap (…180000 / 180001…), so the width
        // is measured from the PREVIOUS band's ceiling to this band's ceiling.
        // Using `to - from` would silently drop one unit of base per band.
        let taxable_in_bracket = match bracket.to {
            Some(bracket_max) => {
                let width = bracket_max - band_floor;
                remaining.min(width).max(Decimal::ZERO)
            }
            None => remaining,
        };

        if taxable_in_bracket > Decimal::ZERO {
            let tax_in_bracket = (taxable_in_bracket * bracket.rate).round_dp_with_strategy(
                DEFAULT_SCALE,
                rust_decimal::RoundingStrategy::MidpointAwayFromZero,
            );

            total_tax += tax_in_bracket;
            remaining -= taxable_in_bracket;

            applications.push(IprBracketApplication {
                bracket_from: bracket.from,
                bracket_to: bracket.to,
                rate: bracket.rate,
                taxable_in_bracket,
                tax_in_bracket,
            });
        }

        // Advance the floor to this band's ceiling for the next iteration.
        if let Some(bracket_max) = bracket.to {
            band_floor = bracket_max;
        }
    }

    let ipr = Money::new(
        total_tax.round_dp_with_strategy(
            DEFAULT_SCALE,
            rust_decimal::RoundingStrategy::MidpointAwayFromZero,
        ),
        config.currency.clone(),
    );

    Ok((ipr, applications))
}

/// Calculate CNSS contributions (employee and employer).
pub fn calculate_cnss(
    gross_for_cnss: Money,
    config: &PayrollConfig,
) -> MoneyResult<(Money, Money, Money)> {
    if gross_for_cnss.currency != config.currency {
        return Err(MoneyError::CurrencyMismatch(
            gross_for_cnss.currency.to_string(),
            config.currency.to_string(),
        ));
    }

    // CNSS base is capped at ceiling
    let cnss_base_amount = gross_for_cnss.amount.min(config.cnss.ceiling);
    let cnss_base = Money::new(cnss_base_amount, config.currency.clone());

    let employee_cnss = cnss_base
        .checked_mul(config.cnss.employee_rate)?
        .rounded(RoundingPolicy::RoundHalfUp);
    let employer_cnss = cnss_base
        .checked_mul(config.cnss.employer_rate)?
        .rounded(RoundingPolicy::RoundHalfUp);

    Ok((cnss_base, employee_cnss, employer_cnss))
}

/// Calculate SMIG threshold for given days worked.
pub fn calculate_smig_threshold(days_worked: u32, config: &PayrollConfig) -> Money {
    let daily_rate = config.smig.daily;
    let threshold_amount = daily_rate * Decimal::from(days_worked);
    Money::new(threshold_amount, config.currency.clone())
}

/// Check if net pay meets SMIG requirement.
pub fn check_smig_compliance(net_pay: Money, days_worked: u32, config: &PayrollConfig) -> bool {
    let smig_threshold = calculate_smig_threshold(days_worked, config);
    net_pay.amount >= smig_threshold.amount
}

/// Main payroll calculation function.
pub fn calculate_payroll(
    input: PayrollInput,
    config: &PayrollConfig,
) -> MoneyResult<PayrollResult> {
    // Validate currency consistency
    if input.gross_pay.currency != config.currency {
        return Err(MoneyError::CurrencyMismatch(
            input.gross_pay.currency.to_string(),
            config.currency.to_string(),
        ));
    }

    // 1. Calculate overtime pay
    let overtime_pay =
        if let (Some(hours), Some(rate)) = (input.overtime_hours, input.overtime_rate) {
            let hourly_rate = input.gross_pay.amount / Decimal::from(173); // ~173 hours/month
            let overtime_amount = hourly_rate * hours * rate;
            Money::new(
                overtime_amount.round_dp_with_strategy(
                    DEFAULT_SCALE,
                    rust_decimal::RoundingStrategy::MidpointAwayFromZero,
                ),
                config.currency.clone(),
            )
        } else {
            Money::zero(config.currency.clone())
        };

    // 2. Sum allowances
    let mut total_allowances = Money::zero(config.currency.clone());
    let mut taxable_allowances = Money::zero(config.currency.clone());
    let mut cnssable_allowances = Money::zero(config.currency.clone());

    for allowance in &input.allowances {
        total_allowances = total_allowances.checked_add(&allowance.amount)?;
        if allowance.is_taxable {
            taxable_allowances = taxable_allowances.checked_add(&allowance.amount)?;
        }
        if allowance.is_cnssable {
            cnssable_allowances = cnssable_allowances.checked_add(&allowance.amount)?;
        }
    }

    // 3. Sum pre-tax deductions
    let mut pre_tax_deductions = Money::zero(config.currency.clone());
    for deduction in &input.deductions {
        pre_tax_deductions = pre_tax_deductions.checked_add(&deduction.amount)?;
    }

    // 4. Calculate taxable gross (for IPR)
    // Taxable gross = gross + taxable allowances - pre-tax deductions
    let taxable_gross = input
        .gross_pay
        .checked_add(&taxable_allowances)?
        .checked_sub(&pre_tax_deductions)?;

    // 5. Calculate IPR
    let (ipr_deduction, ipr_brackets_applied) = calculate_ipr(taxable_gross.clone(), config)?;

    // 6. Calculate CNSS base (gross + cnssable allowances, capped)
    let cnss_gross = input.gross_pay.checked_add(&cnssable_allowances)?;
    let (cnss_base, cnss_employee, cnss_employer) = calculate_cnss(cnss_gross, config)?;

    // 7. Total deductions from employee
    let total_deductions = ipr_deduction
        .checked_add(&cnss_employee)?
        .checked_add(&pre_tax_deductions)?;

    // 8. Net pay
    let net_pay = taxable_gross.checked_sub(&total_deductions)?;

    // 9. Employer cost (gross + employer CNSS + employer contributions)
    let employer_cost = input
        .gross_pay
        .checked_add(&cnss_employer)?
        .checked_add(&total_allowances)?; // Employer bears allowances too

    // 10. SMIG compliance
    let smig_threshold = calculate_smig_threshold(input.days_worked, config);
    let smig_compliant = check_smig_compliance(net_pay.clone(), input.days_worked, config);

    Ok(PayrollResult {
        employee_id: input.employee_id,
        gross_pay: input.gross_pay,
        overtime_pay,
        total_allowances,
        taxable_gross,
        ipr_deduction,
        cnss_employee,
        cnss_employer,
        total_deductions,
        net_pay,
        employer_cost,
        smig_compliant,
        calculation_details: CalculationDetails {
            ipr_brackets_applied,
            cnss_base,
            smig_threshold,
            rounding_policy: RoundingPolicy::RoundHalfUp,
        },
    })
}

/// Calculate payroll run totals across all employees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollRunTotals {
    pub total_gross: Money,
    pub total_net: Money,
    pub total_employer_cost: Money,
    pub employee_count: usize,
    pub smig_compliant_count: usize,
    pub smig_non_compliant_count: usize,
}

pub fn calculate_run_totals(results: &[PayrollResult]) -> MoneyResult<PayrollRunTotals> {
    if results.is_empty() {
        return Ok(PayrollRunTotals {
            total_gross: Money::zero(CurrencyCode::cdf()),
            total_net: Money::zero(CurrencyCode::cdf()),
            total_employer_cost: Money::zero(CurrencyCode::cdf()),
            employee_count: 0,
            smig_compliant_count: 0,
            smig_non_compliant_count: 0,
        });
    }

    let currency = results[0].gross_pay.currency.clone();
    let mut total_gross = Money::zero(currency.clone());
    let mut total_net = Money::zero(currency.clone());
    let mut total_employer_cost = Money::zero(currency.clone());
    let mut smig_compliant_count = 0;
    let mut smig_non_compliant_count = 0;

    for result in results {
        total_gross = total_gross.checked_add(&result.gross_pay)?;
        total_net = total_net.checked_add(&result.net_pay)?;
        total_employer_cost = total_employer_cost.checked_add(&result.employer_cost)?;

        if result.smig_compliant {
            smig_compliant_count += 1;
        } else {
            smig_non_compliant_count += 1;
        }
    }

    Ok(PayrollRunTotals {
        total_gross,
        total_net,
        total_employer_cost,
        employee_count: results.len(),
        smig_compliant_count,
        smig_non_compliant_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_common::money;

    fn test_config() -> PayrollConfig {
        PayrollConfig::default()
    }

    #[test]
    fn test_ipr_calculation_first_bracket_exempt() {
        let config = test_config();
        let income = money!("100000", "CDF"); // Below 180,000 - exempt
        let (ipr, apps) = calculate_ipr(income, &config).unwrap();
        assert_eq!(ipr.amount, Decimal::ZERO);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].rate, Decimal::ZERO);
    }

    #[test]
    fn test_ipr_calculation_second_bracket() {
        let config = test_config();
        let income = money!("300000", "CDF");
        let (ipr, apps) = calculate_ipr(income, &config).unwrap();
        // Progressive: first 180,000 exempt, remaining 120,000 at 15%.
        // The bands are declared with a 1-unit gap (…180000 / 180001…) but the
        // base must not lose that unit, so the slice is 120,000 — not 119,999.
        let expected = Decimal::new(18000, 0);
        assert_eq!(ipr.amount, expected);
        assert_eq!(apps.len(), 2);
        assert_eq!(apps[1].taxable_in_bracket, Decimal::new(120000, 0));
    }

    #[test]
    fn test_ipr_top_bracket_does_not_overflow() {
        // The top band has `to: None`. Taking `Decimal::MAX - from` there would
        // panic on overflow, so the open band must consume whatever remains.
        let config = test_config();
        let income = money!("6000000", "CDF");
        let (ipr, apps) = calculate_ipr(income, &config).unwrap();
        assert!(ipr.amount > Decimal::ZERO);
        assert_eq!(apps.len(), 6, "all six bands should be applied");
        // Full base is consumed: sum of per-band slices == taxable income.
        let consumed: Decimal = apps.iter().map(|a| a.taxable_in_bracket).sum();
        assert_eq!(consumed, Decimal::new(6000000, 0));
        // Top band takes 6,000,000 - 4,800,000 = 1,200,000 at 40%.
        assert_eq!(apps[5].taxable_in_bracket, Decimal::new(1200000, 0));
        assert_eq!(apps[5].tax_in_bracket, Decimal::new(480000, 0));
    }

    #[test]
    fn test_ipr_bands_are_exhaustive_and_ordered() {
        // Guards the invariant the slicing math depends on: bands are sorted and
        // every unit of base lands in exactly one band.
        let config = test_config();
        for w in config.ipr_brackets.windows(2) {
            let prev_to = w[0].to.expect("only the last band may be open");
            assert!(w[1].from > prev_to, "bands must not overlap");
        }
        assert!(
            config.ipr_brackets.last().unwrap().to.is_none(),
            "top band must be open-ended"
        );
    }

    #[test]
    fn test_cnss_calculation_under_ceiling() {
        let config = test_config();
        let gross = money!("500000", "CDF"); // Under 1,500,000 ceiling
        let (base, emp, employer) = calculate_cnss(gross, &config).unwrap();
        assert_eq!(base.amount, Decimal::new(500000, 0));
        assert_eq!(
            emp.amount,
            (Decimal::new(500000, 0) * Decimal::new(5, 2)).round_dp_with_strategy(
                DEFAULT_SCALE,
                rust_decimal::RoundingStrategy::MidpointAwayFromZero
            )
        );
        assert_eq!(
            employer.amount,
            (Decimal::new(500000, 0) * Decimal::new(105, 3)).round_dp_with_strategy(
                DEFAULT_SCALE,
                rust_decimal::RoundingStrategy::MidpointAwayFromZero
            )
        );
    }

    #[test]
    fn test_cnss_calculation_over_ceiling() {
        let config = test_config();
        let gross = money!("2000000", "CDF"); // Over 1,500,000 ceiling
        let (base, emp, employer) = calculate_cnss(gross, &config).unwrap();
        assert_eq!(base.amount, Decimal::new(1500000, 0)); // Capped at ceiling
        assert_eq!(
            emp.amount,
            (Decimal::new(1500000, 0) * Decimal::new(5, 2)).round_dp_with_strategy(
                DEFAULT_SCALE,
                rust_decimal::RoundingStrategy::MidpointAwayFromZero
            )
        );
    }

    #[test]
    fn test_smig_compliance() {
        let config = test_config();
        let net_pay = money!("200000", "CDF"); // Above monthly_26 (182,000)
        assert!(check_smig_compliance(net_pay, 26, &config));

        let low_pay = money!("150000", "CDF"); // Below SMIG
        assert!(!check_smig_compliance(low_pay, 26, &config));
    }

    #[test]
    fn test_full_payroll_calculation() {
        let config = test_config();
        let input = PayrollInput {
            employee_id: uuid::Uuid::new_v4(),
            gross_pay: money!("500000", "CDF"),
            overtime_hours: Some(Decimal::new(10, 0)),
            overtime_rate: Some(Decimal::new(15, 1)), // 1.5x
            allowances: vec![PayrollAllowance {
                code: "HOUSING".to_string(),
                description: "Housing allowance".to_string(),
                amount: money!("100000", "CDF"),
                is_taxable: true,
                is_cnssable: true,
            }],
            deductions: vec![],
            days_worked: 26,
        };

        let result = calculate_payroll(input, &config).unwrap();

        // Basic sanity checks
        assert!(result.gross_pay.amount > Decimal::ZERO);
        assert!(result.overtime_pay.amount > Decimal::ZERO);
        assert!(result.ipr_deduction.amount >= Decimal::ZERO);
        assert!(result.cnss_employee.amount > Decimal::ZERO);
        assert!(result.cnss_employer.amount > Decimal::ZERO);
        assert!(result.net_pay.amount > Decimal::ZERO);
        assert!(result.employer_cost.amount > result.gross_pay.amount);
        assert!(result.smig_compliant); // 500k gross should yield net > 182k SMIG
    }

    #[test]
    fn test_run_totals() {
        let config = test_config();
        let results = vec![
            PayrollResult {
                employee_id: uuid::Uuid::new_v4(),
                gross_pay: money!("500000", "CDF"),
                overtime_pay: money!("0", "CDF"),
                total_allowances: money!("0", "CDF"),
                taxable_gross: money!("500000", "CDF"),
                ipr_deduction: money!("50000", "CDF"),
                cnss_employee: money!("25000", "CDF"),
                cnss_employer: money!("52500", "CDF"),
                total_deductions: money!("75000", "CDF"),
                net_pay: money!("425000", "CDF"),
                employer_cost: money!("552500", "CDF"),
                smig_compliant: true,
                calculation_details: CalculationDetails {
                    ipr_brackets_applied: vec![],
                    cnss_base: money!("500000", "CDF"),
                    smig_threshold: money!("182000", "CDF"),
                    rounding_policy: RoundingPolicy::RoundHalfUp,
                },
            },
            PayrollResult {
                employee_id: uuid::Uuid::new_v4(),
                gross_pay: money!("300000", "CDF"),
                overtime_pay: money!("0", "CDF"),
                total_allowances: money!("0", "CDF"),
                taxable_gross: money!("300000", "CDF"),
                ipr_deduction: money!("18000", "CDF"),
                cnss_employee: money!("15000", "CDF"),
                cnss_employer: money!("31500", "CDF"),
                total_deductions: money!("33000", "CDF"),
                net_pay: money!("267000", "CDF"),
                employer_cost: money!("331500", "CDF"),
                smig_compliant: true,
                calculation_details: CalculationDetails {
                    ipr_brackets_applied: vec![],
                    cnss_base: money!("300000", "CDF"),
                    smig_threshold: money!("182000", "CDF"),
                    rounding_policy: RoundingPolicy::RoundHalfUp,
                },
            },
        ];

        let totals = calculate_run_totals(&results).unwrap();
        assert_eq!(totals.employee_count, 2);
        assert_eq!(totals.total_gross.amount, Decimal::new(800000, 0));
        assert_eq!(totals.total_net.amount, Decimal::new(692000, 0));
        assert_eq!(totals.total_employer_cost.amount, Decimal::new(884000, 0));
        assert_eq!(totals.smig_compliant_count, 2);
        assert_eq!(totals.smig_non_compliant_count, 0);
    }
}
