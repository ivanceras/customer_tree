use datafusion::prelude::*;
//use datafusion::arrow::array::RecordBatch;
use arrow::array::RecordBatch;

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
