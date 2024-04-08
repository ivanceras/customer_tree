use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use flate2::bufread::GzDecoder;
use async_trait::async_trait;
use std::fs::File;
use std::io::BufReader;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use datafusion::prelude::SessionContext;
use gauntlet::DataSource;
use std::io::Cursor;

static DATA: &[u8]  = include_bytes!("../../../data/customer_export.gz");


#[derive(Debug, Serialize, Deserialize)]
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
    log::info!("in customer main..");
    /*
    log::info!("here in async main..");
    let in_file = "../../data/customer_export.gz";
    let in_file = File::open(in_file)?;
    */
    let in_file = Cursor::new(DATA);
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_reader(BufReader::new(GzDecoder::new(BufReader::new(in_file))));

    log::info!("Reading customers data..");
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
        //log::info!("customer: {:#?}", customer);
        customers.push(customer);

    }

    log::info!("Creating a csv..");
    let mut wtr = csv::WriterBuilder::new().has_headers(false).from_writer(vec![]);
    for c in customers.iter().take(10){
        wtr.serialize(c)?;
    }
    log::info!("done writing csv..");

    let header = "customer{eq_id:u64,sponsor_eq_id:u64,parent_eq_id:u64,created_date:utc,change_date:utc,full_name:text,invoice_phone_number:text,delivery_phone_number:text,invoice_address:text,shipping_address:text}";
    let data = format!("{}\n{}",header,String::from_utf8(wtr.into_inner()?)?);
    //log::info!("data: {}", data);
    let customer = DataSource::new(data.as_bytes().to_vec())?;
    let customer_table = customer.into_memtable()?;


    let ctx = SessionContext::new();

    ctx.register_table("customer", Arc::new(customer_table))?;
    //let df = ctx.sql("SELECT eq_id, full_name FROM customer LIMIT 5").await?;
    let df = ctx.sql("SELECT * FROM customer LIMIT 5").await?;
    let result = df.collect().await?;
    log::info!("customer: {:#?}", result);

    Ok(())
}


#[cfg(test)]
mod tests{
    use super::*;
    use chrono::NaiveDate;
    use chrono::NaiveTime;

    #[tokio::test]
    async fn customer(){
        main().await.unwrap();
    }

    #[test]
    fn test_parse_date(){
        let date = format_date("1970-01-01 00:00:00");
        dbg!(&date);
        let nd = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let nt = NaiveTime::from_hms_milli_opt(0,0,0,0).unwrap();
        assert_eq!(date, Some(NaiveDateTime::new(nd, nt)))
    }
}
