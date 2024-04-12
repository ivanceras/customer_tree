use gauntlet::Context;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data_source = customer::customer_data().await?;

    let ctx = Context::new();

    ctx.register_table("customer", data_source)?;
    // top-level customer with no recruiter
    let data = ctx.sql("SELECT * FROM customer WHERE parent_eq_id IS NULL").await?;
    let d1 = ctx.sql("SELECT t1.eq_id, t1.full_name, 
        (SELECT count(eq_id) as children FROM customer t3 WHERE t3.parent_eq_id = t1.eq_id)
        FROM customer t1 ORDER BY children DESC LIMIT 20").await?;
    d1.show()?;

    let d2 = ctx.sql("SELECT t1.eq_id, t1.full_name, 
        (SELECT count(eq_id) as sponsored FROM customer t3 WHERE t3.sponsor_eq_id = t1.eq_id)
        FROM customer t1 ORDER BY sponsored DESC LIMIT 20").await?;
    d2.show()?;
    Ok(())
}
