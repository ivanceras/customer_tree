use arrow::array::RecordBatch;
use csv::ReaderBuilder;
use csv::StringRecord;
use datafusion::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ctx = SessionContext::new();

    ctx.register_csv("cities", "./data/worldcities.csv", CsvReadOptions::new())
        .await?;

    // create a plan
    let df = ctx
        .sql("SELECT city, lat, lng, iso2 FROM cities ORDER BY city ASC LIMIT 100")
        .await?;
    let results: Vec<RecordBatch> = df.collect().await?;

    // format the results
    let pretty_results = arrow::util::pretty::pretty_format_batches(&results)?.to_string();
    println!("{}", pretty_results);
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
