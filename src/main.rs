use gauntlet::Context;
use gauntlet::DataPane;

async fn get_children(ctx: &Context, eq_id: u64) -> anyhow::Result<DataPane>{
    let sql = format!("SELECT eq_id, full_name, parent_eq_id,
                (SELECT COUNT(*) as children 
                    FROM customer t1 
                    WHERE t1.parent_eq_id=customer.eq_id)
        FROM customer 
        WHERE customer.parent_eq_id = {eq_id} 
        ORDER BY children DESC
        LIMIT 5");
    let d1 = ctx.sql(&sql).await?;
    Ok(d1)
}

async fn get_sponsored(ctx: &Context, eq_id: u64) -> anyhow::Result<DataPane>{
    let sql = format!("SELECT eq_id, full_name, parent_eq_id,
                (SELECT COUNT(*) as children 
                    FROM customer t1 
                    WHERE t1.parent_eq_id=customer.eq_id)
        FROM customer 
        WHERE customer.sponsor_eq_id = {eq_id} 
        ORDER BY children DESC
        LIMIT 5");
    let d1 = ctx.sql(&sql).await?;
    Ok(d1)
}

async fn get_sponsored_count(ctx:&Context, eq_id: u64) -> anyhow::Result<i64>{
    let sql = format!("SELECT count(*) AS count
        FROM customer 
        WHERE customer.sponsor_eq_id = {eq_id}");
    let d1 = ctx.sql(&sql).await?;
    let count: i64 = Into::into(&d1.row_values[0][0]);
    Ok(count)
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data_source = customer::customer_data().await?;

    let ctx = Context::new();

    ctx.register_table("customer", data_source)?;
    let count = ctx.sql("SELECT COUNT(*) FROM customer").await?;
    count.show()?;
    // top-level customer with no recruiter
    let data = ctx.sql("SELECT eq_id, full_name, 
                (SELECT COUNT(*) as children 
                    FROM customer t1 
                    WHERE t1.parent_eq_id=customer.eq_id)
                FROM customer 
                ORDER BY children DESC 
                LIMIT 10").await?;

    data.show()?;

    for data in data.row_values{
        let eq_id = Into::into(&data[0]);
        let full_name: String = Into::into(&data[1]);
        let children: u64 = Into::into(&data[2]);
        let sponsored: i64 = get_sponsored_count(&ctx, eq_id).await?;
        println!("{eq_id}, {full_name}, children: {children}, sponsored: {sponsored}");
        let children = get_children(&ctx, eq_id).await?;
        println!("children (top 5):");
        children.show()?;
    }
    Ok(())
}
