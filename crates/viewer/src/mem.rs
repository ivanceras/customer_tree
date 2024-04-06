
use datafusion::arrow::array::{UInt64Array, UInt8Array};
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::datasource::MemTable;
use datafusion::error::Result;
use datafusion::prelude::SessionContext;
use std::sync::Arc;
use std::time::Duration;

/// This example demonstrates executing a simple query against a Memtable
pub async fn main() -> Result<()> {
    let mem_table = create_memtable()?;

    // create local execution context
    let ctx = SessionContext::new();

    // Register the in-memory table containing the data
    ctx.register_table("users", Arc::new(mem_table))?;

    let dataframe = ctx.sql("SELECT * FROM users;").await?;

        let result = dataframe.collect().await.unwrap();
        let record_batch = result.first().unwrap();

        assert_eq!(1, record_batch.column(0).len());
        log::info!("mem table colums: {:#?}", record_batch.columns());

    Ok(())
}

fn create_memtable() -> Result<MemTable> {
    MemTable::try_new(get_schema(), vec![vec![create_record_batch()?,create_record_batch()?]])
}

fn create_record_batch() -> Result<RecordBatch> {
    let id_array = UInt8Array::from(vec![1]);
    let account_array = UInt64Array::from(vec![9000]);

    Ok(RecordBatch::try_new(
        get_schema(),
        vec![Arc::new(id_array), Arc::new(account_array)],
    )
    .unwrap())
}

fn get_schema() -> SchemaRef {
    SchemaRef::new(Schema::new(vec![
        Field::new("id", DataType::UInt8, false),
        Field::new("bank_account", DataType::UInt64, true),
    ]))
}
