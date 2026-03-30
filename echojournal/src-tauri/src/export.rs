use crate::RawEntry;
use genpdf::{elements, style, Element}; // Added Element here
use docx_rs::*;
use std::fs::File;

pub fn generate_pdf(entries: Vec<RawEntry>, path: String) -> Result<(), String> {
    // 1. Read font file
    let font_data = std::fs::read("./font.ttf")
        .map_err(|_| "font.ttf not found in src-tauri. Please add it for PDF export.")?;
    
    let font = genpdf::fonts::FontData::new(font_data, None)
        .map_err(|e| e.to_string())?;

    // 2. Initialize FontFamily directly (this version doesn't use ::new)
    // We use the same font for all styles for the hackathon demo
    let font_family = genpdf::fonts::FontFamily {
        regular: font.clone(),
        bold: font.clone(),
        italic: font.clone(),
        bold_italic: font,
    };
    
    let mut doc = genpdf::Document::new(font_family);
    doc.set_title("EchoJournal Export");

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(20);
    doc.set_page_decorator(decorator);

    let bold_style = style::Style::new().bold();

    for entry in entries {
        // Now .styled() will work because 'Element' is in scope
        doc.push(elements::Text::new(format!("{} | {}", entry.date, entry.time)).styled(bold_style));
        doc.push(elements::Paragraph::new(format!("[#{}] {}", entry.tag, entry.text)));
        doc.push(elements::Break::new(1));
    }

    doc.render_to_file(path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn generate_docx(entries: Vec<RawEntry>, path: String) -> Result<(), String> {
    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut docx = Docx::new();

    for entry in entries {
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(format!("{} - {} ", entry.date, entry.time)).bold())
                .add_run(Run::new().add_text(format!("[#{}] ", entry.tag)).italic())
                .add_run(Run::new().add_text(&entry.text))
        );
    }

    docx.build().pack(file).map_err(|e| e.to_string())?;
    Ok(())
}