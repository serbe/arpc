use actix::Arbiter;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse};
use futures::future::{result, Future};
use serde_json::json;

use crate::messages::{UrlGetterMsg, UrlMsg};
use crate::WebData;

pub fn post_url_list(data: web::Data<WebData>, params: web::Json<Vec<String>>) -> HttpResponse {
    let url_list: Vec<String> = params.into_inner();
    let mut num = 0;
    for url in url_list {
        if url.contains("://") {
            data.manager.do_send(UrlMsg(url));
            num += 1;
        } else {
            data.manager.do_send(UrlMsg(format!("http://{}", url)));
            data.manager.do_send(UrlMsg(format!("socks5://{}", url)));
            num += 2;
        }
    }
    HttpResponse::Ok().json(json!({ "num": num }))
}

pub fn check_type(data: web::Data<WebData>, path: web::Path<(String, i64)>) -> HttpResponse {
    // let data = data.clone();
    // let path = path.clone();
    // // web::block(move || {
    let getter = match path.0.as_str() {
        "anon" => Some(UrlGetterMsg {
            limit: path.1,
            anon: Some(true),
            work: true,
            hours: Some(6),
        }),
        "work" => Some(UrlGetterMsg {
            limit: path.1,
            anon: None,
            work: true,
            hours: Some(6),
        }),
        "all" => Some(UrlGetterMsg {
            limit: path.1,
            anon: None,
            work: false,
            hours: Some(6),
        }),
        _ => None,
    };
    if let Some(getter) = getter {
        let db = data.db.clone();
        let manager = data.manager.clone();
        let response = db.send(getter);
        Arbiter::spawn(response.then(move |list| {
            match list {
                Ok(urls) => {
                    println!("list len {}", urls.len());
                    for url in urls {
                        manager.do_send(UrlMsg(url));
                    }
                }
                Err(e) => println!("error {}", e),
            }

            Arbiter::current().stop();
            result(Ok(()))
        }));
    }
    HttpResponse::new(StatusCode::OK)
}
