use crate::client::{FastGptData, Reference, SearchResult, SummarizeData};

pub fn format_fastgpt(data: &FastGptData) {
    println!("{}", data.output);

    if !data.references.is_empty() {
        println!();
        println!("Sources:");
        for (i, r) in data.references.iter().enumerate() {
            format_reference(i + 1, r);
        }
    }
}

fn format_reference(n: usize, r: &Reference) {
    println!("  {n}. {} — {}", r.title, r.url);
    if !r.snippet.is_empty() {
        println!("     {}", r.snippet);
    }
}

pub fn format_summary(data: &SummarizeData) {
    println!("{}", data.output);
}

pub fn format_web_search(results: &[SearchResult]) {
    for (i, result) in results.iter().enumerate() {
        let date = result.published.as_deref().unwrap_or("");
        let date_str = if date.is_empty() { String::new() } else { format!(" ({date})") };
        println!("  {}. {}{}", i + 1, result.title, date_str);
        println!("     {}", result.url);
        if let Some(snippet) = &result.snippet {
            println!("     {snippet}");
        }
        println!();
    }
}
