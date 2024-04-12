use customer;
use data_viewer::views::{data_view, DataView};
use gauntlet::Context;
use gauntlet::DataPane;
use sauron::*;

#[derive(Debug)]
pub enum Msg {
    ReceiveDataPane(DataPane),
    DataViewMsg(data_view::Msg),
    MouseMove(i32, i32),
    EndResize(i32, i32),
    StartResize(Grip, i32, i32),
    SqlChanged(String),
    ExecuteSql,
    QueryError(gauntlet::Error),
}

/// provides a resizable wrapper for the DataView
#[derive(Default)]
pub struct App {
    data_view: Option<DataView>,
    active_resize: Option<Grip>,
    sql: String,
    query_error: Option<gauntlet::Error>,
    width: i32,
    height: i32,
    start_x: i32,
    start_y: i32,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Grip {
    Right,
    Bottom,
    BottomRight,
}

impl App {
    pub fn new() -> Self {
        App {
            data_view: None,
            active_resize: None,
            sql: "SELECT * FROM customer LIMIT 10".to_string(),
            query_error: None,
            width: 400,
            height: 500,
            start_x: 0,
            start_y: 0,
        }
    }

    fn execute_sql(&mut self) -> Cmd<Msg>{
        let sql = self.sql.clone();
        Cmd::new(async move{
            let ctx = Context::new();
            let data_pane = customer::customer_data().await.unwrap();
            ctx.register_table("customer", data_pane).unwrap();
            match ctx.sql(&sql).await{
                Ok(records) => Msg::ReceiveDataPane(records),
                Err(e) => Msg::QueryError(e),
            }
        })
    }
}

impl Application for App {
    type MSG = Msg;

    /// Setup the resize wrapper to listen to the mouseup
    /// and mousemove event of the Window
    /// to have a continuity and ensure that the mousemouve
    /// event is always captured.
    /// Unliked when listen by the this view container, which the mouse
    /// can be outside of this view, which causes the mousmove event
    /// not being triggered
    fn init(&mut self) -> Cmd<Msg> {
        Cmd::batch([
            Window::on_mouseup(|event| Msg::EndResize(event.client_x(), event.client_y())),
            Window::on_mousemove(|event| Msg::MouseMove(event.client_x(), event.client_y())),
            self.execute_sql(),
        ])
    }

    fn update(&mut self, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::ReceiveDataPane(data_pane) => {
                log::info!("Receiving data pane..");
                let mut data_view = DataView::from_data_pane(data_pane).unwrap();
                let column_widths = [200, 200, 200, 500, 200, 200, 200];
                let total_width = column_widths.iter().fold(0, |acc, cw| acc + cw + 10);
                data_view.set_allocated_size(total_width, 600);
                data_view.set_column_widths(&column_widths);
                let width = data_view.allocated_width;
                let height = data_view.allocated_height;
                self.data_view = Some(data_view);
                self.width = width;
                self.height = height;
                Cmd::none()
            }
            Msg::DataViewMsg(data_view_msg) => {
                if let Some(data_view) = self.data_view.as_mut() {
                    let effects = data_view.update(data_view_msg);
                    Cmd::from(effects.map_msg(Msg::DataViewMsg))
                } else {
                    Cmd::none()
                }
            }
            Msg::EndResize(client_x, client_y) => {
                self.active_resize = None;
                if let Some(data_view) = self.data_view.as_mut() {
                    let effects =
                        data_view.update(data_view::Msg::ColumnEndResize(client_x, client_y));
                    Cmd::from(effects.map_msg(Msg::DataViewMsg))
                } else {
                    Cmd::none()
                }
            }
            Msg::MouseMove(client_x, client_y) => {
                if let Some(active_resize) = &self.active_resize {
                    match active_resize {
                        Grip::BottomRight => {
                            let delta_x = client_x - self.start_x;
                            let delta_y = client_y - self.start_y;
                            self.width += delta_x;
                            self.height += delta_y;
                            self.start_x = client_x;
                            self.start_y = client_y;
                        }
                        Grip::Right => {
                            let delta_x = client_x - self.start_x;
                            self.width += delta_x;
                            self.start_x = client_x;
                        }
                        Grip::Bottom => {
                            let delta_y = client_y - self.start_y;
                            self.height += delta_y;
                            self.start_y = client_y;
                        }
                    }
                    if let Some(data_view) = self.data_view.as_mut() {
                        data_view.set_allocated_size(self.width, self.height);
                    }
                }
                if let Some(data_view) = self.data_view.as_mut() {
                    let effects = data_view.update(data_view::Msg::MouseMove(client_x, client_y));
                    Cmd::from(effects.map_msg(Msg::DataViewMsg))
                } else {
                    Cmd::none()
                }
            }
            Msg::StartResize(grip, client_x, client_y) => {
                self.active_resize = Some(grip);
                self.start_x = client_x;
                self.start_y = client_y;
                Cmd::none()
            }
            Msg::SqlChanged(sql) => {
                self.sql = sql;
                Cmd::none()
            }
            Msg::ExecuteSql => {
                self.query_error = None;
                self.execute_sql()
            }
            Msg::QueryError(e) => {
                self.query_error = Some(e);
                Cmd::none()
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        main([class("app")],
            [
                div(
                    [],
                    [textarea(
                        [style!{
                            display: "inline-block",
                            width: px(1000),
                            height: px(200),
                        },
                        on_change(|e|Msg::SqlChanged(e.value())),
                        ],
                        [text(&self.sql)],
                    ),
                    button([on_click(|_|Msg::ExecuteSql)],[text("Execute SQL")]),
                    if let Some(error) = self.query_error.as_ref(){
                        div([], [text!("{error}")])
                    }else{
                        span([],[])
                    },
                    ],
                ),
                div(
                    [class("resize_wrapper grid")],
                    [
                        if let Some(data_view) = &self.data_view {
                            data_view.view().map_msg(Msg::DataViewMsg)
                        } else {
                            log::info!("loading..");
                            div([], [text("Loading..")])
                        },
                        div(
                            [
                                class("resize_wrapper__resize_grip resize_wrapper__resize_grip--right"),
                                on_mousedown(|event| {
                                    Msg::StartResize(Grip::Right, event.client_x(), event.client_y())
                                }),
                            ],
                            [],
                        ),
                        div(
                            [
                                class("resize_wrapper__resize_grip resize_wrapper__resize_grip--bottom"),
                                on_mousedown(|event| {
                                    Msg::StartResize(Grip::Bottom, event.client_x(), event.client_y())
                                }),
                            ],
                            [],
                        ),
                        div(
                            [
                                class(
                                    "resize_wrapper__resize_grip resize_wrapper__resize_grip--bottom_right",
                                ),
                                on_mousedown(|event| {
                                    Msg::StartResize(Grip::BottomRight, event.client_x(), event.client_y())
                                }),
                            ],
                            [],
                        ),
                        a(
                            [href(
                                "https://github.com/ivanceras/sauron/tree/master/examples/data-viewer/",
                            )],
                            [text("code")],
                        ),
                    ],
                )
            ])
    }

    fn stylesheet() -> Vec<String> {
        vec![jss! {
            "body": {
                font_family: "Fira Sans, Courier New, Courier, Lucida Sans Typewriter, Lucida Typewriter, monospace",
            }
        }]
    }
}
