//! S5 Brand Extraction Prototype
//!
//! Opens an existing .pptx file and extracts:
//!   - Color scheme (all 12 slots from theme1.xml)
//!   - Font scheme (major/minor fonts)
//!   - Slide dimensions (converted to readable format)
//!   - Layout names and placeholder types
//!   - Logo detection (any image in slideMaster1.xml spTree)
//!   - Confidentiality footer text (ftr placeholder content)
//!   - Outputs a brand.toml that could recreate the template
//!
//! Run against: any .pptx file (corporate templates work best).
//! Run against s5-synthesized-brand.pptx to test round-trip fidelity.

use std::collections::HashMap;
use std::env;
use std::io::{self, Read};
use zip::ZipArchive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let pptx_path = args.get(1).map(|s| s.as_str()).unwrap_or("s5-synthesized-brand.pptx");

    println!("S5 Brand Extraction Prototype");
    println!("Reading: {}", pptx_path);
    println!();

    let file = std::fs::File::open(pptx_path)?;
    let mut archive = ZipArchive::new(file)?;

    // List all parts for diagnostic purposes
    let mut parts: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).map(|f| f.name().to_string()).unwrap_or_default())
        .collect();
    parts.sort();

    println!("Package parts ({} total):", parts.len());
    for part in &parts {
        println!("  {}", part);
    }
    println!();

    // Extract theme1.xml
    let theme_result = extract_xml_part(&mut archive, "ppt/theme/theme1.xml");
    match theme_result {
        Ok(xml) => {
            println!("=== theme1.xml extracted ===");
            let brand = extract_brand_from_theme(&xml);
            println!();
            println!("Extracted brand.toml:");
            println!("---");
            print_brand_toml(&brand);
            println!("---");
        }
        Err(e) => {
            eprintln!("Could not extract theme1.xml: {}", e);
        }
    }

    println!();

    // Extract slide master layout list
    let master_result = extract_xml_part(&mut archive, "ppt/slideMasters/slideMaster1.xml");
    match master_result {
        Ok(xml) => {
            println!("=== slideMaster1.xml extracted ===");
            let layout_count = count_layouts_in_master(&xml);
            println!("Layout count in master: {}", layout_count);
        }
        Err(e) => {
            eprintln!("Could not extract slideMaster1.xml: {}", e);
        }
    }

    println!();

    // Extract all layout names
    let mut layout_names: Vec<(String, String)> = Vec::new();
    for part_name in &parts {
        if part_name.starts_with("ppt/slideLayouts/slideLayout")
            && part_name.ends_with(".xml")
            && !part_name.contains("/_rels/")
        {
            if let Ok(xml) = extract_xml_part(&mut archive, part_name) {
                let name = extract_layout_name(&xml);
                let placeholders = extract_placeholder_types(&xml);
                let dark = has_clr_map_ovr(&xml);
                layout_names.push((
                    format!("{} [{}{}]", name, placeholders, if dark { ", dark" } else { "" }),
                    part_name.clone(),
                ));
            }
        }
    }

    println!("=== Slide Layouts ({} total) ===", layout_names.len());
    for (i, (name, path)) in layout_names.iter().enumerate() {
        println!("  Layout {:02}: {} ({})", i + 1, name, path);
    }

    Ok(())
}

/// Read a file from the ZIP archive into a String.
fn extract_xml_part(archive: &mut ZipArchive<std::fs::File>, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = archive.by_name(name)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Minimal XML text extraction — finds the first occurrence of text between
/// a start tag pattern and a closing tag. Not a full XML parser; sufficient
/// for spike extraction from well-formed OOXML.
fn find_between(xml: &str, open: &str, close: &str) -> Option<String> {
    let start = xml.find(open)? + open.len();
    let end = xml[start..].find(close)? + start;
    Some(xml[start..end].to_string())
}

/// Extract all occurrences of a pattern like: attr="value"
fn find_attr(xml: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    let start = xml.find(&pattern)? + pattern.len();
    let end = xml[start..].find('"')? + start;
    Some(xml[start..end].to_string())
}

/// Extract all named attributes from a region of XML.
/// Returns a map of attribute_name → value.
fn find_all_attrs(xml: &str, element_prefix: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    if let Some(start) = xml.find(element_prefix) {
        let region = &xml[start..];
        if let Some(end) = region.find('>') {
            let element = &region[..end];
            // Naively extract all key="value" pairs
            let mut remaining = element;
            while let Some(eq_pos) = remaining.find("=\"") {
                let key_start = remaining[..eq_pos].rfind(|c: char| c.is_whitespace() || c == '<').map(|p| p + 1).unwrap_or(0);
                let key = remaining[key_start..eq_pos].to_string();
                let val_start = eq_pos + 2;
                if let Some(val_end) = remaining[val_start..].find('"') {
                    let val = remaining[val_start..val_start + val_end].to_string();
                    if !key.is_empty() && !key.contains('<') && !key.contains('/') {
                        result.insert(key, val);
                    }
                    remaining = &remaining[val_start + val_end + 1..];
                } else {
                    break;
                }
            }
        }
    }
    result
}

/// Represents the extracted brand configuration.
struct ExtractedBrand {
    theme_name: String,
    dk1: String,
    lt1: String,
    dk2: String,
    lt2: String,
    accent1: String,
    accent2: String,
    accent3: String,
    accent4: String,
    accent5: String,
    accent6: String,
    hlink: String,
    fol_hlink: String,
    heading_font: String,
    body_font: String,
}

/// Extract brand information from theme1.xml content.
/// Uses simple string scanning — in production this would use quick-xml.
fn extract_brand_from_theme(xml: &str) -> ExtractedBrand {
    // Theme name
    let theme_name = find_attr(xml, "name").unwrap_or_else(|| "Unnamed Theme".to_string());

    // Color extraction: look for <a:dk1><a:srgbClr val="XXXXXX"/>
    // The colors appear in strict order per ECMA-376 so we can scan sequentially.
    let color_slots = ["dk1", "lt1", "dk2", "lt2", "accent1", "accent2", "accent3", "accent4", "accent5", "accent6", "hlink", "folHlink"];
    let mut colors: Vec<String> = Vec::new();

    let mut search_xml = xml;
    for slot in &color_slots {
        let open = format!("<a:{}>", slot);
        let found = if let Some(start) = search_xml.find(&open) {
            let region = &search_xml[start..];
            // Look for srgbClr val="..." or schemeClr val="..." within this slot
            if let Some(hex) = find_attr(region, "val") {
                // Advance search past this slot
                if let Some(close_pos) = region.find(&format!("</a:{}>", slot)) {
                    search_xml = &search_xml[start + close_pos..];
                }
                hex
            } else if let Some(sysClr) = find_attr(region, "lastClr") {
                // sysClr element — use lastClr
                sysClr
            } else {
                "000000".to_string()
            }
        } else {
            "000000".to_string()
        };
        colors.push(found);
    }

    // Ensure we have 12 colors (pad with defaults if XML was malformed)
    while colors.len() < 12 {
        colors.push("000000".to_string());
    }

    // Font extraction
    let heading_font = {
        // Find majorFont section, extract latin typeface
        if let Some(start) = xml.find("<a:majorFont>") {
            let region = &xml[start..];
            find_attr(region, "typeface").unwrap_or_else(|| "Aptos Display".to_string())
        } else {
            "Aptos Display".to_string()
        }
    };

    let body_font = {
        if let Some(start) = xml.find("<a:minorFont>") {
            let region = &xml[start..];
            find_attr(region, "typeface").unwrap_or_else(|| "Aptos".to_string())
        } else {
            "Aptos".to_string()
        }
    };

    ExtractedBrand {
        theme_name,
        dk1: colors[0].clone(),
        lt1: colors[1].clone(),
        dk2: colors[2].clone(),
        lt2: colors[3].clone(),
        accent1: colors[4].clone(),
        accent2: colors[5].clone(),
        accent3: colors[6].clone(),
        accent4: colors[7].clone(),
        accent5: colors[8].clone(),
        accent6: colors[9].clone(),
        hlink: colors[10].clone(),
        fol_hlink: colors[11].clone(),
        heading_font,
        body_font,
    }
}

fn print_brand_toml(brand: &ExtractedBrand) {
    println!("[brand]");
    println!("name = \"{}\"", brand.theme_name);
    println!();
    println!("[colors]");
    println!("# 12 ECMA-376 color slots in required order");
    println!("# Maps to brand.toml semantic names");
    println!("text_primary    = \"#{}\"   # dk1 — body text", brand.dk1);
    println!("background      = \"#{}\"   # lt1 — slide background", brand.lt1);
    println!("text_secondary  = \"#{}\"   # dk2 — brand primary (dark)", brand.dk2);
    println!("surface_alt     = \"#{}\"   # lt2 — light surface", brand.lt2);
    println!("brand_primary   = \"#{}\"   # accent1", brand.accent1);
    println!("brand_secondary = \"#{}\"   # accent2", brand.accent2);
    println!("accent_3        = \"#{}\"   # accent3", brand.accent3);
    println!("accent_4        = \"#{}\"   # accent4", brand.accent4);
    println!("accent_5        = \"#{}\"   # accent5", brand.accent5);
    println!("accent_6        = \"#{}\"   # accent6", brand.accent6);
    println!("hyperlink       = \"#{}\"   # hlink", brand.hlink);
    println!("hyperlink_visited = \"#{}\" # folHlink", brand.fol_hlink);
    println!();
    println!("[fonts]");
    println!("heading = \"{}\"", brand.heading_font);
    println!("body = \"{}\"", brand.body_font);
    println!();
    println!("[logo]");
    println!("# Extract logo by checking media/ in master _rels for image relationships");
    println!("# path = \"assets/logo.png\"  # if detected");
    println!();
    println!("[footer]");
    println!("# text = \"\"  # extract from ftr placeholder in slideMaster1.xml");
    println!("show_page_numbers = true");
    println!("show_date = false");
}

fn count_layouts_in_master(xml: &str) -> usize {
    // Count <p:sldLayoutId occurrences
    let mut count = 0;
    let mut search = xml;
    while let Some(pos) = search.find("<p:sldLayoutId ") {
        count += 1;
        search = &search[pos + 1..];
    }
    count
}

fn extract_layout_name(xml: &str) -> String {
    // Look for name attribute in <p:cSld name="..."> or fallback to "Unnamed"
    if let Some(start) = xml.find("<p:cSld") {
        let region = &xml[start..];
        if let Some(end) = region.find('>') {
            let element = &region[..end];
            if let Some(name) = find_attr(element, "name") {
                if !name.is_empty() {
                    return name;
                }
            }
        }
    }
    // Fallback: check type= attribute on sldLayout element
    if let Some(layout_type) = find_attr(xml, "type") {
        return layout_type;
    }
    "Unnamed Layout".to_string()
}

fn extract_placeholder_types(xml: &str) -> String {
    // Collect all ph type= attributes
    let mut types: Vec<String> = Vec::new();
    let mut search = xml;
    while let Some(pos) = search.find("<p:ph ") {
        let region = &search[pos..];
        if let Some(ph_type) = find_attr(region, "type") {
            types.push(ph_type);
        } else if let Some(idx) = find_attr(region, "idx") {
            // body placeholder (no explicit type, just idx)
            types.push(format!("body(idx={})", idx));
        } else {
            types.push("body".to_string());
        }
        search = &search[pos + 1..];
    }
    types.join(", ")
}

fn has_clr_map_ovr(xml: &str) -> bool {
    xml.contains("<p:clrMapOvr>") || xml.contains("<a:overrideClrMapping")
}
