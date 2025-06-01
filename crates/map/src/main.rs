use arrow::array::ArrayRef;
use arrow::array::Float64Array;
use arrow::array::RecordBatch;
use arrow::array::StringArray;
use arrow::datatypes::DataType;
use arrow::datatypes::Field;
use arrow::datatypes::Fields;
use arrow::datatypes::Schema;
use csv::ReaderBuilder;
use csv::StringRecord;
use csv::Terminator;
use datafusion::common::cast::as_float64_array;
use datafusion::common::cast::as_string_array;
use datafusion::logical_expr_common::signature::Volatility;
use datafusion::physical_plan::ColumnarValue;
use datafusion::prelude::*;
use std::sync::Arc;
use datafusion::datasource::file_format::file_compression_type::FileCompressionType;

const USE_GZ: bool = true;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ctx = SessionContext::new();

    let mut csv_options = CsvReadOptions::new();

    if USE_GZ {
        println!("Using the compressed gz file");
        csv_options.file_extension = "gz";
        csv_options.file_compression_type = FileCompressionType::GZIP;
        ctx.register_csv("customer", "./data/customer_export.gz", csv_options)
            .await?;
    } else {
        println!("using the csv file..");
        ctx.register_csv("customer", "./data/customer_export.csv", csv_options)
            .await?;
    }

    ctx.register_csv("cities", "./data/cities.csv", CsvReadOptions::new())
        .await?;

    let extract_city = Arc::new(|args: &[ColumnarValue]| {
        assert_eq!(args.len(), 1);
        let args = ColumnarValue::values_to_arrays(args)?;
        let base = as_string_array(&args[0]).expect("cast failed");
        fn get_city(addr: &str) -> Option<String> {
            let mut rdr = ReaderBuilder::new()
                .has_headers(false)
                .terminator(Terminator::Any(0))
                .from_reader(addr.as_bytes());

            let records: Vec<StringRecord> = rdr
                .records()
                .map(|r| match r {
                    Ok(r) => r,
                    Err(e) => {
                        dbg!(&addr);
                        panic!("error here: {e}")
                    }
                })
                .collect();
            assert_eq!(records.len(), 1);
            records[0].iter().rev().nth(2).map(|s| s.trim().to_string())
        }
        let array = base
            .iter()
            .map(|base| match base {
                Some(base) => get_city(base),
                _ => None,
            })
            .collect::<StringArray>();
        Ok(ColumnarValue::from(Arc::new(array) as ArrayRef))
    });

    let extract_city = create_udf(
        "extract_city",
        vec![DataType::Utf8],
        DataType::Utf8,
        Volatility::Immutable,
        extract_city,
    );
    ctx.register_udf(extract_city);

    let extract_country = Arc::new(|args: &[ColumnarValue]| {
        assert_eq!(args.len(), 1);
        let args = ColumnarValue::values_to_arrays(args)?;
        let base = as_string_array(&args[0]).expect("cast failed");
        fn get_country(addr: &str) -> Option<String> {
            let mut rdr = ReaderBuilder::new()
                .has_headers(false)
                .terminator(Terminator::Any(0))
                .from_reader(addr.as_bytes());

            let records: Vec<StringRecord> = rdr
                .records()
                .map(|r| match r {
                    Ok(r) => r,
                    Err(e) => {
                        dbg!(&addr);
                        panic!("error here: {e}")
                    }
                })
                .collect();
            assert_eq!(records.len(), 1);
            records[0].iter().rev().next().map(|s| s.trim().to_string())
        }
        let array = base
            .iter()
            .map(|base| match base {
                Some(base) => get_country(base),
                _ => None,
            })
            .collect::<StringArray>();
        Ok(ColumnarValue::from(Arc::new(array) as ArrayRef))
    });

    let extract_country = create_udf(
        "extract_country",
        vec![DataType::Utf8],
        DataType::Utf8,
        Volatility::Immutable,
        extract_country,
    );
    ctx.register_udf(extract_country);

    let df = ctx
        .sql(
            "WITH t1 AS
            (SELECT full_name, extract_city(shipping_address) AS city, extract_country(shipping_address) AS country
                FROM customer
            )
            SELECT t1.*, cities.latitude, cities.longitude from t1
                LEFT JOIN cities
                    ON cities.name = t1.city
                    AND cities.country_code = t1.country
            ORDER BY full_name ASC
            LIMIT 1000000
            ",
        )
        .await?;
    df.show().await.unwrap();

    Ok(())
}

#[test]
fn test1() {
    let addr = "Raadhuisstraat (2401231509), Amsterdam, 3036, NO";
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(addr.as_bytes());

    let records: Vec<StringRecord> = rdr.records().map(|r| r.unwrap()).collect();
    assert_eq!(records.len(), 1);
    dbg!(&records);
    let city = records[0].iter().rev().nth(2).unwrap().trim();
    dbg!(&city);
    panic!()
}
