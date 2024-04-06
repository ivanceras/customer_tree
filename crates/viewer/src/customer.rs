use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use datafusion::arrow::array::{UInt64Builder, UInt8Builder};
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::common::arrow::array::UInt64Array;
use datafusion::common::arrow::array::UInt8Array;
use datafusion::dataframe::DataFrame;
use datafusion::datasource::{provider_as_source, TableProvider, TableType};
use datafusion::error::Result;
use datafusion::execution::context::{SessionState, TaskContext};
use datafusion::physical_expr::EquivalenceProperties;
use datafusion::physical_plan::memory::MemoryStream;
use datafusion::physical_plan::{
    project_schema, DisplayAs, DisplayFormatType, ExecutionMode, ExecutionPlan, Partitioning,
    PlanProperties, SendableRecordBatchStream,
};
use datafusion::prelude::*;
use datafusion_expr::{Expr, LogicalPlanBuilder};
use flate2::bufread::GzDecoder;
use async_trait::async_trait;
use std::fs::File;
use std::io::BufReader;
use serde::{Deserialize, Serialize};
use datafusion::common::arrow::array::StringArray;
use datafusion::common::arrow::array::StringBuilder;
use datafusion::common::arrow::array::Date64Array;
use datafusion::common::arrow::array::Date64Builder;
use chrono::NaiveDateTime;


#[derive(Debug)]
struct Customer{
    eq_id: Option<u64>,
    sponsor_eq_id: Option<u64>,
    parent_eq_id: Option<u64>,
    created_date: Option<NaiveDateTime>,
    change_date: Option<NaiveDateTime>,
    full_name: String,
    invoice_phone_number: String,
    delivery_phone_number: String,
    invoice_address: String,
    shipping_address: String,
}


fn format_date(date: &str) -> Option<NaiveDateTime> {
    let dt = NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S");
    dt.ok()
}


/// This example demonstrates executing a simple query against a custom datasource
pub(crate) async fn main() -> anyhow::Result<()> {
    log::info!("here in async main..");
    let in_file = "../../data/customer_export.gz";
    let in_file = File::open(in_file)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_reader(BufReader::new(GzDecoder::new(BufReader::new(in_file))));

    let mut customers = vec![];
    for (i, result) in rdr.records().enumerate() {
        let result = result?;
        let eq_id = result[0].parse().ok();
        let sponsor_eq_id = result[1].parse().ok();
        let parent_eq_id = result[2].parse().ok();
        let created_date = format_date(&result[3]);
        let change_date = format_date(&result[4]);
        let full_name = result[5].to_string();
        let invoice_phone_number = result[6].to_string();
        let delivery_phone_number = result[7].to_string();
        let invoice_address = result[8].to_string();
        let shipping_address = result[9].to_string();

        let customer =  Customer{
            eq_id,
            sponsor_eq_id,
            parent_eq_id,
            created_date,
            change_date,
            full_name,
            invoice_phone_number,
            delivery_phone_number,
            invoice_address,
            shipping_address,
        };
        customers.push(customer);

    }

    let schema = Schema::new(vec![
        Field::new("eq_id", DataType::UInt64, true),
        Field::new("sponsor_eq_id", DataType::UInt64, true),
        Field::new("parent_eq_id", DataType::UInt64, true),
        Field::new("created_date", DataType::Date64, true),
        Field::new("change_date", DataType::Date64, true),
        Field::new("full_name", DataType::Utf8, true),
        Field::new("invoice_phone_number", DataType::Utf8, true),
        Field::new("delivery_phone_number", DataType::Utf8, true),
        Field::new("invoice_address", DataType::Utf8, true),
        Field::new("shipping_address", DataType::Utf8, true),
    ]);
    let db = CustomDataSource::new(customers, schema);

    println!("executing sql here..");
    let ctx = SessionContext::new();

    ctx.register_table("customer", Arc::new(db.clone()))?;
    let df = ctx.sql("SELECT * FROM customer LIMIT 10").await?;
    let result = df.collect().await?;
    log::info!("customer: {:#?}", result);

    Ok(())
}

/// A custom datasource, used to represent a datastore with a single index
#[derive(Clone)]
pub struct CustomDataSource {
    inner: Arc<Mutex<CustomDataSourceInner>>,
    schema: SchemaRef,
}

struct CustomDataSourceInner {
    eq_id: Arc<UInt64Array>,
    sponsor_eq_id: Arc<UInt64Array>,
    parent_eq_id: Arc<UInt64Array>,
    created_date: Arc<Date64Array>,
    change_date: Arc<Date64Array>,
    full_name: Arc<StringArray>,
    invoice_phone_number: Arc<StringArray>,
    delivery_phone_number: Arc<StringArray>,
    invoice_address: Arc<StringArray>,
    shipping_address: Arc<StringArray>,
}

impl Debug for CustomDataSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("custom_db")
    }
}

impl CustomDataSource {
    pub fn new(customers: Vec<Customer>, schema: Schema) -> Self {
        let mut eq_id = UInt64Builder::new();
        let mut sponsor_eq_id = UInt64Builder::new();
        let mut parent_eq_id = UInt64Builder::new();
        let mut created_date = Date64Builder::new();
        let mut change_date = Date64Builder::new();
        let mut full_name = StringBuilder::new();
        let mut invoice_phone_number = StringBuilder::new();
        let mut delivery_phone_number = StringBuilder::new();
        let mut invoice_address = StringBuilder::new();
        let mut shipping_address = StringBuilder::new();

        for c in customers {
             eq_id.append_option(c.eq_id);
             sponsor_eq_id.append_option(c.sponsor_eq_id);
             parent_eq_id.append_option(c.parent_eq_id);
             created_date.append_option(c.created_date.map(|d|d.and_utc().timestamp_millis())); 
             change_date.append_option(c.change_date.map(|d|d.and_utc().timestamp_millis())); 
             full_name.append_value(c.full_name); 
             invoice_phone_number.append_value(c.invoice_phone_number);
             delivery_phone_number.append_value(c.delivery_phone_number); 
             invoice_address.append_value(c.invoice_address); 
             shipping_address.append_value(c.shipping_address); 
        }
        CustomDataSource {
            inner: Arc::new(Mutex::new(CustomDataSourceInner {
                eq_id: Arc::new(eq_id.finish()),
                sponsor_eq_id: Arc::new(sponsor_eq_id.finish()),
                parent_eq_id: Arc::new(parent_eq_id.finish()),
                created_date: Arc::new(created_date.finish()),
                change_date: Arc::new(change_date.finish()),
                full_name: Arc::new(full_name.finish()),
                invoice_phone_number: Arc::new(invoice_phone_number.finish()),
                delivery_phone_number: Arc::new(delivery_phone_number.finish()),
                invoice_address: Arc::new(invoice_address.finish()),
                shipping_address: Arc::new(shipping_address.finish()),
            })),
            schema: SchemaRef::new(schema),
        }
    }

    pub(crate) async fn create_physical_plan(
        &self,
        projections: Option<&Vec<usize>>,
        schema: SchemaRef,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        Ok(Arc::new(CustomExec::new(projections, schema, self.clone())))
    }
}

#[async_trait]
impl TableProvider for CustomDataSource {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        _state: &SessionState,
        projection: Option<&Vec<usize>>,
        // filters and limit can be used here to inject some push-down operations if needed
        _filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        return self.create_physical_plan(projection, self.schema()).await;
    }
}

#[derive(Debug, Clone)]
struct CustomExec {
    db: CustomDataSource,
    projected_schema: SchemaRef,
    cache: PlanProperties,
}

impl CustomExec {
    fn new(projections: Option<&Vec<usize>>, schema: SchemaRef, db: CustomDataSource) -> Self {
        let projected_schema = project_schema(&schema, projections).unwrap();
        let cache = Self::compute_properties(projected_schema.clone());
        Self {
            db,
            projected_schema,
            cache,
        }
    }

    /// This function creates the cache object that stores the plan properties such as schema, equivalence properties, ordering, partitioning, etc.
    fn compute_properties(schema: SchemaRef) -> PlanProperties {
        let eq_properties = EquivalenceProperties::new(schema);
        PlanProperties::new(
            eq_properties,
            Partitioning::UnknownPartitioning(1),
            ExecutionMode::Bounded,
        )
    }
}

impl DisplayAs for CustomExec {
    fn fmt_as(&self, _t: DisplayFormatType, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "CustomExec")
    }
}

impl ExecutionPlan for CustomExec {
    fn name(&self) -> &'static str {
        "CustomExec"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn properties(&self) -> &PlanProperties {
        &self.cache
    }

    fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
        vec![]
    }

    fn with_new_children(
        self: Arc<Self>,
        _: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        Ok(self)
    }

    fn execute(
        &self,
        _partition: usize,
        _context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let db = self.db.inner.lock().unwrap();

        Ok(Box::pin(MemoryStream::try_new(
            vec![RecordBatch::try_new(
                self.projected_schema.clone(),
                vec![
                    db.eq_id.clone(),
                    db.sponsor_eq_id.clone(),
                    db.parent_eq_id.clone(),
                    db.created_date.clone(),
                    db.change_date.clone(),
                    db.full_name.clone(),
                    db.invoice_phone_number.clone(),
                    db.delivery_phone_number.clone(),
                    db.invoice_address.clone(),
                    db.shipping_address.clone(),
                ],
            )?],
            self.schema(),
            None,
        )?))
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use chrono::NaiveDate;
    use chrono::NaiveTime;

    #[test]
    fn test_parse_date(){
        let date = format_date("1970-01-01 00:00:00");
        dbg!(&date);
        let nd = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let nt = NaiveTime::from_hms_milli_opt(0,0,0,0).unwrap();
        assert_eq!(date, Some(NaiveDateTime::new(nd, nt)))
    }
}
