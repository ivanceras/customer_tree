use gauntlet::Context;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data_source = customer::customer_data().await?;

    let ctx = Context::new();

    ctx.register_table("customer", data_source)?;
    let data = ctx.sql("SELECT * FROM customer").await?;
    println!("{}", data);
    Ok(())
}
