use polars::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

pub struct LazyFramer {
    pub lazyframe: Option<LazyFrame>,
    pub columns: Vec<String>,
}

impl LazyFramer {
    pub fn new(files: Vec<PathBuf>) -> Self {
        let files_arc: Arc<[PathBuf]> = Arc::from(files);
        let args = ScanArgsParquet::default();
        log::info!("Files {:?}", files_arc);

        match LazyFrame::scan_parquet_files(files_arc, args) {
            Ok(lf) => {
                log::info!("Loaded Parquet files");
                let column_names = Self::get_column_names_from_lazyframe(&lf);

                Self {
                    lazyframe: Some(lf),
                    columns: column_names,
                }
            }
            Err(e) => {
                log::error!("Failed to load Parquet files: {}", e);
                Self {
                    lazyframe: None, // Indicates that loading failed
                    columns: Vec::new(),
                }
            }
        }
    }

    pub fn set_lazyframe(&mut self, lazyframe: LazyFrame) {
        self.lazyframe = Some(lazyframe);
    }

    pub fn get_column_names(&self) -> Vec<String> {
        self.columns.clone()
    }

    // Adjusted signature to match context
    pub fn get_column_names_from_lazyframe(lazyframe: &LazyFrame) -> Vec<String> {
        let lf: LazyFrame = lazyframe.clone().limit(1);
        let df: DataFrame = lf.collect().unwrap();
        let columns: Vec<String> = df
            .get_column_names_owned()
            .into_iter()
            .map(|name| name.to_string())
            .collect();

        columns
    }

    pub fn save_lazyframe(&mut self, output_path: &PathBuf) -> Result<(), PolarsError> {
        if let Some(ref lf) = self.lazyframe {
            let mut df = lf.clone().collect()?;

            // Open a file in write mode at the specified output path
            let file = File::create(output_path)
                .map_err(|e| PolarsError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

            // Write the filtered DataFrame to a Parquet file
            ParquetWriter::new(file)
                .set_parallel(true)
                .finish(&mut df)?;
        }
        Ok(())
    }
}
