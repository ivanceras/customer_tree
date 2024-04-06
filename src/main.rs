use datafusion::datasource::file_format::file_compression_type::FileCompressionType;
use datafusion::prelude::*;
use flate2::bufread::GzDecoder;
use flate2::bufread::GzEncoder;
use flate2::bufread::ZlibDecoder;
use flate2::bufread::ZlibEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    eq_id: String,
    sponsor_eq_id: String,
    parent_eq_id: String,
    created_date: String,
    change_date: String,
    full_name: String,
    invoice_phone_number: String,
    delivery_phone_number: String,
    invoice_address: String,
    shipping_address: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Customer{
    eq_id: u64,
    sponsor_eq_id: u64,
    parent_eq_id: u64,
    created_date: String,
    change_date: String,
    full_name: String,
    invoice_phone_number: String,
    delivery_phone_number: String,
    invoice_address: String,
    shipping_address: String,
}

impl Record {

    fn into_customer(&self) -> Customer{
        Customer{
            eq_id: self.eq_id.parse().unwrap(),
            sponsor_eq_id: self.sponsor_eq_id.parse().unwrap(),
            parent_eq_id: self.parent_eq_id.parse().unwrap(),
            created_date: self.created_date.to_string(),
            change_date: self.change_date.to_string(),
            full_name: self.full_name.to_string(),
            invoice_phone_number: self.invoice_phone_number.to_string(),
            delivery_phone_number: self.delivery_phone_number.to_string(),
            invoice_address: self.invoice_address.to_string(),
            shipping_address: self.shipping_address.to_string(),
        }
    }

    fn correct_date(&mut self){
        let posix_time = "1970-01-01 00:00:00".to_string();
        if self.created_date == "0000-00-00 00:00:00"
            || self.created_date == "NULL" {
                self.created_date = posix_time.clone();
        }

        if self.change_date == "0000-00-00 00:00:00"
            || self.change_date == "NULL" {
                self.change_date = posix_time;
        }
    }

    fn null_to_empty(&mut self){
        if self.eq_id == "NULL"{
            self.eq_id = "".to_string()
        }
        if self.sponsor_eq_id == "NULL"{
            self.sponsor_eq_id = "".to_string()
        }
        if self.parent_eq_id == "NULL"{
            self.parent_eq_id = "".to_string()
        }
        if self.created_date == "NULL" {
            unreachable!();
        }
        if self.change_date == "NULL"{
            unreachable!();
        }
        if self.full_name == "NULL"{
            self.full_name = "".to_string()
        }
        if self.invoice_phone_number == "NULL"{
            self.invoice_phone_number = "".to_string()
        }
        if self.delivery_phone_number == "NULL"{
            self.delivery_phone_number = "".to_string()
        }
        if self.invoice_address == "NULL" {
            self.invoice_address = "".to_string()
        }
        if self.shipping_address == "NULL"{
            self.shipping_address = "".to_string()
        }
    }
}

fn compress_csv(in_file: &str, out_file: &str) -> anyhow::Result<()> {
    let in_file = File::open(in_file)?;
    let mut z = GzEncoder::new(BufReader::new(in_file), Compression::best());
    let mut file_buffer = BufWriter::new(File::create(out_file)?);
    let mut buffer = vec![];
    z.read_to_end(&mut buffer)?;
    file_buffer.write_all(&buffer)?;
    file_buffer.flush()?;
    Ok(())
}

fn read_compressed_csv(in_file: &str) -> anyhow::Result<()> {
    let in_file = File::open(in_file)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_reader(BufReader::new(GzDecoder::new(BufReader::new(in_file))));

    for (i, result) in rdr.deserialize().enumerate() {
        let record: Record = result?;
    }
    Ok(())
}

/// exclude data that has invalid date
fn correct_csv_data(in_file: &str, out_file: &str) -> anyhow::Result<()> {
    let in_file = File::open(in_file)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_reader(BufReader::new(GzDecoder::new(BufReader::new(in_file))));

    let out_file = File::create(out_file)?;
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_writer(BufWriter::new(out_file));
    for (i, result) in rdr.deserialize().enumerate() {
        let mut record: Record = result?;
            record.correct_date();
            record.null_to_empty();
            wtr.serialize(record)?;
    }
    wtr.flush()?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    compress_csv("data/customer_export.csv", "data/customer_export.gz")?;
    read_compressed_csv("data/customer_export.gz")?;
    let in_file = "data/customer_export.gz";
    let out_file = "out/customer_export_corrected.csv";
    correct_csv_data(in_file, out_file)?;
    compress_csv(out_file, "out/customer_export_corrected.gz.csv")?;
    let ctx = SessionContext::new();
    let df = ctx
        .read_csv(
            "out/customer_export_corrected.gz.csv",
            CsvReadOptions::new().file_compression_type(FileCompressionType::GZIP),
        )
        .await?;
    let schema = df.schema();
    println!("schema: {:#?}", schema);
    df.show().await?;
    Ok(())
}
