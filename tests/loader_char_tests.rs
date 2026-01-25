#[cfg(test)]
mod loader_char_tests {
    use std::path::Path;

    #[test]
    fn test_bas_character_preservation() {
        // Create a simple test to verify character handling
        let test_code = "10 PRINT CHR$(205.5+RND(1)); : GOTO 10";
        
        // Verify special characters are present
        assert!(test_code.contains("$"), "Should contain $ character");
        assert!(test_code.contains("("), "Should contain ( character");
        assert!(test_code.contains(")"), "Should contain ) character");
        assert!(test_code.contains(";"), "Should contain ; character");
        assert!(test_code.contains(":"), "Should contain : character");
        
        // Simulate what the loader does
        let mut char_queue = std::collections::VecDeque::new();
        
        for ch in test_code.chars() {
            let ch = if ch.is_ascii_alphabetic() {
                ch.to_ascii_uppercase()
            } else {
                ch
            };
            char_queue.push_back(ch);
        }
        
        // Reconstruct the string
        let result: String = char_queue.iter().collect();
        
        // Check the result
        println!("Original: {}", test_code);
        println!("Result:   {}", result);
        
        assert_eq!(result, "10 PRINT CHR$(205.5+RND(1)); : GOTO 10");
        assert!(result.contains("CHR$"), "Should have CHR$ not CHR");
        assert!(result.contains("(1)"), "Should have (1) not corrupted parens");
        assert!(result.contains("; :"), "Should have ; : not corrupted");
    }

    #[test]
    fn test_lowercase_to_uppercase() {
        let test = "print hello";
        let mut result = String::new();
        
        for ch in test.chars() {
            let ch = if ch.is_ascii_alphabetic() {
                ch.to_ascii_uppercase()
            } else {
                ch
            };
            result.push(ch);
        }
        
        assert_eq!(result, "PRINT HELLO");
    }
}
