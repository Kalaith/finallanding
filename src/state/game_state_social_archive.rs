use super::*;

pub(crate) fn write_social_archive_markdown(
    history: &[SocialHistoryEntry],
) -> Result<PathBuf, String> {
    let output_dir = PathBuf::from("docs").join("exports");
    std::fs::create_dir_all(&output_dir)
        .map_err(|error| format!("Could not create {}: {}", output_dir.display(), error))?;
    let output_path = output_dir.join("social_archive.md");
    std::fs::write(&output_path, social_archive_markdown(history))
        .map_err(|error| format!("Could not write {}: {}", output_path.display(), error))?;
    Ok(output_path)
}

pub(crate) fn social_archive_markdown(history: &[SocialHistoryEntry]) -> String {
    let mut output = String::from("# The Final Landing Social Archive\n\n");
    output.push_str(&format!("Reports: {}\n\n", history.len()));

    for entry in history.iter().rev() {
        output.push_str(&format!("## Day {}: {}\n\n", entry.day, entry.title));
        output.push_str(&format!(
            "- Mood: {:.0}\n- Relationship: {:+.0}\n- Close pairs: {}\n- Strained pairs: {}\n\n",
            entry.average_mood, entry.average_relationship, entry.close_pairs, entry.strained_pairs
        ));
        output.push_str(&format!("{}\n\n", entry.detail));
        output.push_str(&format!("Recommendation: {}\n\n", entry.recommendation));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_archive_markdown_exports_latest_report_first() {
        let history = vec![
            SocialHistoryEntry::new(
                1,
                "Early friction",
                "Alice and Fiona need space.",
                "Use Apart before the next work block.",
                46.0,
                -8.0,
                0,
                1,
            ),
            SocialHistoryEntry::new(
                2,
                "Shared meal",
                "Bob and Diana stabilized dinner.",
                "Keep the supportive pair together.",
                62.0,
                12.0,
                1,
                0,
            ),
        ];

        let export = social_archive_markdown(&history);

        assert!(export.contains("# The Final Landing Social Archive"));
        assert!(export.contains("Reports: 2"));
        assert!(export.find("Day 2").unwrap() < export.find("Day 1").unwrap());
        assert!(export.contains("Recommendation: Keep the supportive pair together."));
    }
}
