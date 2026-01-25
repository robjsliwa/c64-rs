use std::path::Path;

// Test the loader module
#[cfg(test)]
mod loader_integration_tests {
    use super::*;

    #[test]
    fn test_load_prg_file() {
        let prg_path = Path::new("assets/prg/monopole.prg");
        
        // Check file exists
        assert!(
            prg_path.exists(),
            "PRG test file not found: {:?}",
            prg_path
        );

        // The actual loading and running would require full emulator setup
        // For now, we just verify the file can be opened
        let data = std::fs::read(prg_path).expect("Failed to read PRG file");
        assert!(data.len() > 2, "PRG file should have at least load address");

        // Verify it has a load address
        let load_addr = u16::from_le_bytes([data[0], data[1]]);
        println!("PRG load address: ${:04X}", load_addr);
        assert!(
            load_addr > 0,
            "PRG should have a valid load address"
        );
    }

    #[test]
    fn test_load_bas_file() {
        let bas_path = Path::new("assets/bas/10print.bas");
        
        // Check file exists
        assert!(
            bas_path.exists(),
            "BAS test file not found: {:?}",
            bas_path
        );

        // Verify it's readable text
        let text = std::fs::read_to_string(bas_path).expect("Failed to read BAS file");
        assert!(text.contains("PRINT"), "BAS file should contain BASIC code");
        println!("BAS file content:\n{}", text);
    }

    #[test]
    fn test_all_bas_files_readable() {
        let bas_files = [
            "assets/bas/10print.bas",
            "assets/bas/bitmap.bas",
            "assets/bas/bouncing.bas",
            "assets/bas/lowercase.bas",
            "assets/bas/simple_tone.bas",
            "assets/bas/sound_test.bas",
        ];

        for bas_file in &bas_files {
            let path = Path::new(bas_file);
            assert!(path.exists(), "BAS file not found: {}", bas_file);
            
            let text = std::fs::read_to_string(path)
                .expect(&format!("Failed to read: {}", bas_file));
            
            assert!(!text.is_empty(), "{} should not be empty", bas_file);
            println!("✓ {} is readable ({} bytes)", bas_file, text.len());
        }
    }
}
