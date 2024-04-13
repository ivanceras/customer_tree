use gauntlet::Context;
use gauntlet::DataPane;

async fn get_children(ctx: &Context, eq_id: u64) -> anyhow::Result<DataPane>{
    let sql = format!("SELECT eq_id, full_name, parent_eq_id 
        FROM customer 
        WHERE customer.parent_eq_id = {eq_id} LIMIT 5");
    let d1 = ctx.sql(&sql).await?;
    Ok(d1)
}

async fn get_sponsored(ctx: &Context, eq_id: u64) -> anyhow::Result<DataPane>{
    let sql = format!("SELECT eq_id, full_name, parent_eq_id 
        FROM customer 
        WHERE customer.sponsor_eq_id = {eq_id} LIMIT 5");
    let d1 = ctx.sql(&sql).await?;
    Ok(d1)
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data_source = customer::customer_data().await?;

    let ctx = Context::new();

    ctx.register_table("customer", data_source)?;
    let count = ctx.sql("SELECT COUNT(*) FROM customer").await?;
    count.show()?;
    // top-level customer with no recruiter
    let data = ctx.sql("SELECT *, 
                (SELECT COUNT(*) as children 
                    FROM customer t1 
                    WHERE t1.parent_eq_id=customer.eq_id)
                FROM customer 
                ORDER BY children DESC 
                LIMIT 10").await?;

    for data in data.row_values{
        let eq_id = Into::into(&data[0]);
        let children = get_children(&ctx, eq_id).await?;
        println!("children:");
        children.show()?;
        let sponsored = get_sponsored(&ctx, eq_id).await?;
        println!("sponsored:");
        sponsored.show()?;
    }
    Ok(())
}
