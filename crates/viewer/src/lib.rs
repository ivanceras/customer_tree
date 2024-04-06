
// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.
extern crate wasm_bindgen;

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

mod customer;
mod mem;

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Info).unwrap();
    console_error_panic_hook::set_once();
    basic_exprs();
    basic_parse();
    spawn_local(async move{
        customer::main().await.unwrap();
        mem::main().await.unwrap();
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


#[cfg(test)]
mod test{
    use super::*;

    #[tokio::test]
    async fn test1(){
        pretty_env_logger::init();
        log::info!("logging check..");
        customer::main().await.unwrap();
    }
}
