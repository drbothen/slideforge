// S2 Spike — veraPDF CI Integration Sample
//
// Demonstrates the test helper pattern for PDF/UA-1 validation using veraPDF CLI.
// Throwaway code — for illustrative and proof-of-concept purposes only.
// Production implementation lives in crates/slideforge-pdf/tests/pdfua1.rs (Phase 4).
//
// Prerequisites:
//   macOS: brew install verapdf
//   Linux: apt-get install -y verapdf (or use the veraPDF/veraPDF-greenfield Docker image)
//
// The production CI job uses the veraPDF Docker image directly:
//   docker run --rm -v $(pwd):/pdf verapdf/verapdf:latest --flavour ua1 /pdf/output.pdf

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::process::Command;

    /// Validates `pdf_bytes` against PDF/UA-1 using the veraPDF CLI.
    ///
    /// # Panics
    /// Panics with a detailed veraPDF report if validation fails.
    /// Panics with an installation hint if `verapdf` is not found on PATH.
    pub fn assert_pdfua1_valid(pdf_bytes: &[u8]) {
        // Write PDF to a temp file — veraPDF requires a file path argument
        let mut tmp = tempfile::Builder::new()
            .suffix(".pdf")
            .tempfile()
            .expect("failed to create temp file");
        tmp.write_all(pdf_bytes)
            .expect("failed to write PDF bytes to temp file");
        let path = tmp.path().to_str().expect("temp file path is not UTF-8");

        let output = Command::new("verapdf")
            .args([
                "--flavour", "ua1",
                "--format", "text",
                "--verbosity", "0",
                path,
            ])
            .output()
            .expect(
                "failed to run verapdf — is it installed?\n\
                 macOS: brew install verapdf\n\
                 Linux: apt-get install verapdf\n\
                 Docker: docker run veraPDF/veraPDF:latest"
            );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // veraPDF text output includes "compliant: true" for passing documents
        let is_compliant = stdout.contains("compliant: true")
            || stdout.contains("isCompliant>true<")
            || output.status.success();

        if !is_compliant {
            panic!(
                "PDF failed PDF/UA-1 validation (veraPDF)\n\
                 \n\
                 --- veraPDF stdout ---\n\
                 {stdout}\n\
                 --- veraPDF stderr ---\n\
                 {stderr}"
            );
        }
    }

    /// Placeholder test showing the integration pattern.
    ///
    /// In Phase 4, this would be replaced by a real slideforge-pdf render call.
    ///
    /// Example: generate a 1-slide deck with a title slide and verify it passes
    /// PDF/UA-1 validation.
    #[test]
    #[ignore = "requires veraPDF on PATH and slideforge-pdf render implementation (Phase 4)"]
    fn title_slide_pdf_passes_pdfua1() {
        // Placeholder: in Phase 4, replace with:
        //   let deck = LaidOutDeck::fixture_title_slide();
        //   let pdf_bytes = render_pdf(&deck).expect("PDF render failed");
        //   assert_pdfua1_valid(&pdf_bytes);
        let pdf_bytes: Vec<u8> = vec![];
        assert_pdfua1_valid(&pdf_bytes);
    }

    /// Integration test: all 31 slide types must produce PDF/UA-1 compliant output.
    ///
    /// This is the key CI gate for the PDF exporter's accessibility requirement.
    #[test]
    #[ignore = "requires veraPDF on PATH and slideforge-pdf render implementation (Phase 4)"]
    fn all_slide_types_pdf_passes_pdfua1() {
        // In Phase 4, replace with:
        //   let deck = LaidOutDeck::fixture_all_slide_types();
        //   let pdf_bytes = render_pdf(&deck).expect("PDF render failed");
        //   assert_pdfua1_valid(&pdf_bytes);
        todo!("Phase 4 implementation")
    }
}
