use overlook::attachments::AttachmentProcessor;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_pdf>", args[0]);
        std::process::exit(1);
    }

    let pdf_path = &args[1];
    println!("Testing PDF extraction for: {}", pdf_path);

    match AttachmentProcessor::process_file(pdf_path) {
        Ok(attachment) => {
            println!("\n✅ SUCCESS!");
            println!("File name: {}", attachment.file_name);
            println!("MIME type: {}", attachment.mime_type);

            match &attachment.data {
                overlook::types::AttachmentData::Text { content } => {
                    println!("Extracted text length: {} characters", content.len());
                    println!("\n--- First 500 characters ---");
                    println!("{}", content.chars().take(500).collect::<String>());
                    if content.len() > 500 {
                        println!("... (truncated)");
                    }
                }
                overlook::types::AttachmentData::Image { base64: _ } => {
                    println!("Unexpected: got image data for PDF");
                }
            }
        }
        Err(e) => {
            println!("\n❌ ERROR: {}", e);
        }
    }
}