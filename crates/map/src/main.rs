use arrow::array::ArrayRef;
use arrow::array::Float64Array;
use arrow::array::RecordBatch;
use arrow::array::StringArray;
use arrow::datatypes::DataType;
use csv::ReaderBuilder;
use csv::StringRecord;
use datafusion::common::cast::as_float64_array;
use datafusion::common::cast::as_string_array;
use datafusion::logical_expr_common::signature::Volatility;
use datafusion::physical_plan::ColumnarValue;
use datafusion::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ctx = SessionContext::new();
    ctx.register_csv(
        "customer",
        "./data/customer_sample.csv",
        CsvReadOptions::new(),
    )
    .await?;

    ctx.register_csv("cities", "./data/worldcities.csv", CsvReadOptions::new())
        .await?;

    let extract_city = Arc::new(|args: &[ColumnarValue]| {
        assert_eq!(args.len(), 1);
        let args = ColumnarValue::values_to_arrays(args)?;
        let base = as_string_array(&args[0]).expect("cast failed");
        fn get_city(addr: &str) -> Option<String> {
            let mut rdr = ReaderBuilder::new()
                .has_headers(false)
                .from_reader(addr.as_bytes());

            let records: Vec<StringRecord> = rdr.records().map(|r| r.unwrap()).collect();
            assert_eq!(records.len(), 1);
            let city = records[0].iter().rev().nth(2).map(|s| s.trim().to_string());
            city
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
    ctx.register_udf(extract_city.clone());

    // create a plan
    let df = ctx
        .sql("SELECT city, lat, lng, iso2, extract_city(2) AS power FROM cities ORDER BY city ASC LIMIT 100")
        .await?;
    df.clone().show().await.unwrap();

    let df = ctx
        .sql("WITH t1 AS
            (SELECT full_name, extract_city(shipping_address) AS city_addr
                FROM customer
            )
            SELECT t1.*, cities.lat, cities.lng from t1
                LEFT JOIN cities ON cities.city = t1.city_addr
            ")
        .await?;
    df.clone().show().await.unwrap();

    let results: Vec<RecordBatch> = df.collect().await?;

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
