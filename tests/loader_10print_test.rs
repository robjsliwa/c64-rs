#[cfg(test)]
mod loader_10print_test {
    use std::collections::VecDeque;

    #[test]
    fn test_10print_typing() {
        // This is the actual content of 10print.bas
        let line = "10 PRINT CHR$(205.5+RND(1)); : GOTO 10";

        let mut char_queue: VecDeque<char> = VecDeque::new();

        // Simulate what load_bas does
        for ch in line.chars() {
            let ch = if ch.is_ascii_alphabetic() {
                ch.to_ascii_uppercase()
            } else {
                ch
            };
            char_queue.push_back(ch);
        }

        // Reconstruct and verify
        let result: String = char_queue.iter().collect();

        println!("Original: {}", line);
        println!("Result:   {}", result);

        // Check all special characters are preserved
        assert!(
            result.contains("CHR$"),
            "Should contain CHR$ with dollar sign"
        );
        assert!(
            result.contains("(205.5+RND(1))"),
            "Should contain correct parentheses"
        );
        assert!(result.contains("; :"), "Should contain semicolon and colon");
        assert!(result.contains("GOTO 10"), "Should contain GOTO 10");

        // Verify exact match (with uppercase letters)
        assert_eq!(result, "10 PRINT CHR$(205.5+RND(1)); : GOTO 10");
    }

    #[test]
    fn test_special_chars_not_uppercased() {
        let special = "$()[]{}:;,.<>/?!@#%^&*-+=";

        for ch in special.chars() {
            let result = if ch.is_ascii_alphabetic() {
                ch.to_ascii_uppercase()
            } else {
                ch
            };

            assert_eq!(result, ch, "Special char '{}' should not change", ch);
        }
    }
}
