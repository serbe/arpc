use actix::Addr;
use actix_web::{web, HttpResponse};
use serde_json::json;
use actix_web::http::StatusCode;

use crate::manager::Manager;
use crate::messages::UrlMsg;

pub fn post_url_list(
    manager: web::Data<Addr<Manager>>,
    params: web::Json<Vec<String>>,
) -> HttpResponse {
    let url_list: Vec<String> = params.into_inner();
    let mut num = 0;
    for url in url_list {
        if url.contains("://") {
            manager.do_send(UrlMsg(url));
            num += 1;
        } else {
            manager.do_send(UrlMsg(format!("http://{}", url)));
            manager.do_send(UrlMsg(format!("socks5://{}", url)));
            num += 2;
        }
    }
    HttpResponse::Ok().json(json!({ "num": num }))
}

pub fn check_type(
    manager: web::Data<Addr<Manager>>,
    path: web::Path<String>,
) -> HttpResponse {
    
    HttpResponse::new(StatusCode::OK)
}