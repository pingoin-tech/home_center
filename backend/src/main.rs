use std::time::Duration;

use actix_files;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use backend::{
    devices::{init_sensor_list, SENSOR_LIST},
    eventhandler::{get_event_handler, Handler, EVENT_HANDLER},
    mqtt,
    store::{init_store, STORE},
};
use fireplace::eventhandler::EventType;
use rumqttc::QoS;
use tokio::{self, task, time::sleep};

#[tokio::main]
async fn main() {
    init_store();
    init_sensor_list();

    let (client, eventloop) = mqtt::init("rumqtt-async", "192.168.178.110", 1883);

    client
        .subscribe("shellies/#", QoS::AtMostOnce)
        .await
        .unwrap();

    {
        EVENT_HANDLER
            .lock()
            .expect("could not lock")
            .get_or_insert(Handler::new(client).await);
    }

    task::spawn(mqtt::work(eventloop));

    {
        if let Some(handler) = EVENT_HANDLER.lock().expect("locking failed").as_mut() {
            handler.init_devices().await;
        }
    }
    let http_handler = HttpServer::new(|| {
        App::new()
            .service(trigger_action)
            .service(devices)
            .service(links)
            .service(actix_files::Files::new("/", "./dist/").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))
    .unwrap()
    .run();

    let (_http, _) = tokio::join!(http_handler, event_handler_loop());
}

#[get("/api/devices")]
async fn devices() -> impl Responder {
    match SENSOR_LIST.lock() {
        Ok(mut list_option) => {
            if let Some(list) = list_option.as_mut() {
                list.sort_by(|a, b| b.id.cmp(&a.id));
                return HttpResponse::Ok().json(list);
            }
        }
        Err(_) => {}
    }
    HttpResponse::Ok().body("bla")
}

#[get("/api/links")]
async fn links() -> impl Responder {
    match STORE.lock() {
        Ok(mut store_option) => {
            if let Some(store) = store_option.as_mut() {
                return HttpResponse::Ok().json(&store.config.extra_links);
            }
        }
        Err(_) => {}
    }
    HttpResponse::Ok().body("bla")
}

#[post("/api/trigger-action")]
async fn trigger_action(data: web::Json<EventType>) -> impl Responder {
    println!("{:?}", &data.0);

    let result = get_event_handler(|handler| handler.force_action(data.0), false);

    if result {
        HttpResponse::Ok().body("true")
    } else {
        HttpResponse::Ok().body("false")
    }
}

async fn event_handler_loop() {
    loop {
        if let Some(handler) = EVENT_HANDLER.lock().expect("locking failed").as_mut() {
            handler.work().await;
        }
        sleep(Duration::from_millis(100)).await;
    }
}
