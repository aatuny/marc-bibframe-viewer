use std::collections::HashMap;
use std::{fs::File, path::Path, path::PathBuf};

use tokio::sync::Semaphore;

use bf_viewer_core::MarcIndexEntry;

pub struct AppState {
    pub by_key: HashMap<String, usize>,
    pub file: File,
    pub index: Vec<MarcIndexEntry>,
    pub xsl_dir: PathBuf,
    pub semaphore: tokio::sync::Semaphore,
}

impl AppState {
    pub fn from_env(max_conversions: usize) -> Result<Self, bf_viewer_core::Error> {
        let xml_path = std::env::var("XML_PATH").expect("XML_PATH was not set");
        let index_path = std::env::var("XML_INDEX_PATH").expect("XML_INDEX_PATH was not set");
        let xsl_dir = PathBuf::from(std::env::var("XSL_PATH").expect("XSL_PATH was not set"));

        // Verify xsl sheets are found from path
        for name in ["ConvSpec-Preprocess0-Splitting.xsl", "marc2bibframe2.xsl"] {
            let p = &xsl_dir.join(name);
            if !p.exists() {
                panic!("stylesheet not found: {}", p.display());
            }
        }

        // Verify xsltproc is installed by utilizing --version
        if std::process::Command::new("xsltproc")
            .arg("--version")
            .output()
            .is_err()
        {
            panic!("xsltproc not found on PATH");
        }

        let marc_file = File::open(&xml_path)?;
        let index: Vec<MarcIndexEntry> = bf_viewer_core::read_index(Path::new(&index_path))?;

        // Build index at startup so that no scan is needed when using record number
        // This allows O(1) for seeking index by using record number with minor RAM cost
        let by_key = index
            .iter()
            .enumerate()
            .map(|(i, e)| (e.key.clone(), i))
            .collect();

        Ok(Self {
            index,
            file: marc_file,
            by_key,
            semaphore: Semaphore::new(max_conversions),
            xsl_dir,
        })
    }
}
