//! Payslip PDF generation.
//!
//! Minimal hand-rolled A4 PDF writer. No external PDF crate — keeps the build
//! hermetic and lets us ship a working payslip without runtime PDF dependencies.
//!
//! Produces a valid PDF-1.4 document with a single page, Helvetica body text,
//! and all monetary amounts rendered via [`Money::to_canonical_string()`]
//! (Decimal string, never float).

use chrono::NaiveDate;
use nexora_common::money::Money;
use std::io::Write;

/// Minimal data required to render one payslip page.
pub struct PayslipPdfData {
    pub gross_pay: Money,
    pub net_pay: Money,
    pub cnss_employee: Money,
    pub cnss_employer: Money,
    pub ipr_deduction: Money,
    pub employer_cost: Money,
    pub employee_name: String,
    pub employee_number: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub config_version: String,
}

/// Generate a payslip PDF for a single employee.
///
/// Returns the raw PDF bytes ready for the HTTP response body or for upload to
/// object storage as the payslip's `object_key`.
pub fn generate_payslip_pdf(data: &PayslipPdfData) -> Result<Vec<u8>, PdfError> {
    let mut buf = Vec::with_capacity(4096);

    // PDF header (19 bytes).
    buf.write_all(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n")
        .map_err(|_| PdfError)?;

    // Object offsets for xref table (computed as we build).
    let off1 = 19u32;

    // Object 1: Catalog.
    buf.write_all(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n")
        .map_err(|_| PdfError)?;
    let off2 =
        (off1 as usize + b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".len()) as u32;

    // Object 2: Pages.
    buf.write_all(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n")
        .map_err(|_| PdfError)?;
    let off3 = (off2 as usize
        + b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n".len())
        as u32;

    // Object 3: Page.
    let obj3 = b"3 0 obj\n\
         << /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842]\
            /Contents 4 0 R\
            /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n";
    buf.write_all(obj3).map_err(|_| PdfError)?;
    let off4 = (off3 as usize + obj3.len()) as u32;

    // Object 4: Content stream.
    let stream = build_content_stream(data)?;
    let stream_len = stream.len() as i64;
    let obj4_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", stream_len);
    buf.write_all(&obj4_hdr.as_bytes()).map_err(|_| PdfError)?;
    buf.write_all(&stream).map_err(|_| PdfError)?;
    buf.write_all(b"endstream\nendobj\n")
        .map_err(|_| PdfError)?;
    let off5 =
        (off4 as usize + obj4_hdr.len() + stream.len() + b"endstream\nendobj\n".len()) as u32;

    // Object 5: Font.
    let obj5 = b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n";
    buf.write_all(obj5).map_err(|_| PdfError)?;

    // Xref table.
    buf.write_all(b"xref\n0 6\n").map_err(|_| PdfError)?;
    buf.write_all(b"0000000000 65535 f \n")
        .map_err(|_| PdfError)?;
    write_xref_entry(&mut buf, off1)?;
    write_xref_entry(&mut buf, off2)?;
    write_xref_entry(&mut buf, off3)?;
    write_xref_entry(&mut buf, off4)?;
    write_xref_entry(&mut buf, off5)?;

    // Trailer.
    let xref_offset = buf.len() as u32;
    let trailer = format!(
        "trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        xref_offset
    );
    buf.write_all(&trailer.as_bytes()).map_err(|_| PdfError)?;

    Ok(buf)
}

fn write_xref_entry(buf: &mut Vec<u8>, offset: u32) -> Result<(), PdfError> {
    let s = format!("{:010} 00000 n \n", offset);
    buf.write_all(&s.as_bytes()).map_err(|_| PdfError)
}

/// Build the page content stream: text positioned on an A4 page.
fn build_content_stream(data: &PayslipPdfData) -> Result<Vec<u8>, PdfError> {
    let mut c = Vec::new();

    // Begin text.
    c.write_all(b"BT\n").map_err(|_| PdfError)?;
    c.write_all(b"/F1 10 Tf\n").map_err(|_| PdfError)?;

    let y_start = 760;
    let mut y = y_start;

    // Title.
    c.extend_from_slice(format!("150 {}\nTd\n", y).as_bytes());
    c.write_all(b"(Nexora Payroll Services) Tj\n")
        .map_err(|_| PdfError)?;
    y -= 25;

    // Employer / config line.
    c.extend_from_slice(format!("150 {}\nTd\n", y).as_bytes());
    c.extend_from_slice(format!("Config: {}", data.config_version).as_bytes());
    c.write_all(b" Tj\n").map_err(|_| PdfError)?;
    y -= 20;

    // Employee line.
    c.extend_from_slice(format!("150 {}\nTd\n", y).as_bytes());
    let emp_line = format!(
        "Employee: {} ({})",
        data.employee_name, data.employee_number
    );
    c.extend_from_slice(emp_line.as_bytes());
    c.write_all(b" Tj\n").map_err(|_| PdfError)?;
    y -= 20;

    // Period line.
    c.extend_from_slice(format!("150 {}\nTd\n", y).as_bytes());
    let period_line = format!(
        "Period: {} to {}",
        data.period_start.format("%Y-%m-%d"),
        data.period_end.format("%Y-%m-%d")
    );
    c.extend_from_slice(period_line.as_bytes());
    c.write_all(b" Tj\n").map_err(|_| PdfError)?;
    y -= 30;

    // Line items.
    let items = [
        ("Gross Pay", &data.gross_pay),
        ("Net Pay", &data.net_pay),
        ("CNSS Employee", &data.cnss_employee),
        ("CNSS Employer", &data.cnss_employer),
        ("IPR Deduction", &data.ipr_deduction),
        ("Employer Cost", &data.employer_cost),
    ];

    for (label, value) in &items {
        c.extend_from_slice(format!("150 {}\nTd\n", y).as_bytes());
        c.extend_from_slice(label.as_bytes());
        c.write_all(b" Tj\n").map_err(|_| PdfError)?;

        c.extend_from_slice(format!("380 {}\nTd\n", y).as_bytes());
        c.extend_from_slice(value.to_canonical_string().as_bytes());
        c.write_all(b" Tj\n").map_err(|_| PdfError)?;

        y -= 18;
    }

    // End text.
    c.write_all(b"ET\n").map_err(|_| PdfError)?;

    Ok(c)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfError;

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PDF generation failed")
    }
}

impl std::error::Error for PdfError {}
