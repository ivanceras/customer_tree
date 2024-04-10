#![deny(warnings)]

use datafusion::common::{DFSchema, ScalarValue};
use datafusion_expr::execution_props::ExecutionProps;
use datafusion_expr::lit;
use datafusion_expr::simplify::SimplifyContext;
use datafusion::optimizer::simplify_expressions::ExprSimplifier;
use datafusion::sql::sqlparser::dialect::GenericDialect;
use datafusion::sql::sqlparser::parser::Parser;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use app::App;
use error::Error;
use sauron::Program;
use data_viewer::views::DataView;

mod customer;
mod app;
mod error;

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Info).unwrap();
    console_error_panic_hook::set_once();
    basic_exprs();
    basic_parse();
    log::info!("attempting to spawn..");
    spawn_local(async move{
        let data_pane = customer::customer_data().await.unwrap();
        let mut data_view = DataView::from_data_pane(data_pane).unwrap();

        //let column_widths = [200, 200, 200, 500, 200];
        //let total_width = column_widths.iter().fold(0, |acc, cw| acc + cw + 10);
        data_view.set_allocated_size(1000, 600);
        //data_view.set_column_widths(&column_widths);
        let width = data_view.allocated_width;
        let height = data_view.allocated_height;
        Program::mount_to_body(App::new(data_view, width, height));
    });
}


#[wasm_bindgen]
pub fn basic_exprs() {
    // Create a scalar value (from datafusion-common)
    let scalar = ScalarValue::from("Hello, World!");
    log::info!("ScalarValue: {scalar:?}");

    // Create an Expr (from datafusion-expr)
    let expr = lit(28) + lit(72);
    log::info!("Expr: {expr:?}");

    // Simplify Expr (using datafusion-phys-expr and datafusion-optimizer)
    let schema = Arc::new(DFSchema::empty());
    let execution_props = ExecutionProps::new();
    let simplifier =
        ExprSimplifier::new(SimplifyContext::new(&execution_props).with_schema(schema));
    let simplified_expr = simplifier.simplify(expr).unwrap();
    log::info!("Simplified Expr: {simplified_expr:?}");
}

#[wasm_bindgen]
pub fn basic_parse() {
    // Parse SQL (using datafusion-sql)
    let sql = "SELECT 2 + 37";
    let dialect = GenericDialect {}; // or AnsiDialect, or your own dialect ...
    let ast = Parser::parse_sql(&dialect, sql).unwrap();
    log::info!("Parsed SQL: {ast:?}");
}


