use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use serde_json;

use embedded_svc::{http::Headers, io::Write, ipv4::IpInfo};

use crate::{cmd::Cmd, gcode::GcodeParser, gimbal::Gimbal};

use {
    esp_idf_svc::http::{server::EspHttpServer, Method},
    log::info,
};

pub fn start(
    ip_info: IpInfo,
    _state: Arc<Mutex<VecDeque<Cmd>>>,
    gimbal_arc: Arc<Mutex<Gimbal>>,
) -> anyhow::Result<EspHttpServer<'static>> {
    let ip = ip_info.ip;
    info!("starting server at {ip}");
    let location = std::format!("https://cdaringe.github.io/gimbal-gui?gimbal_url={ip}");

    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: 10240,
        ..Default::default()
    };

    let mut server = EspHttpServer::new(&server_configuration)?;

    server.fn_handler("/", Method::Get, move |req| {
        let conn = req.connection().unwrap_or("unknown");
        info!("handling req from connection: {conn}");
        let mut response = req.into_response(301, None, &[("Location", &location)])?;
        response.flush()?;
        Ok(())
    })?;

    server.fn_handler("/api/state", Method::Get, move |req| {
        let mut response = req.into_response(
            200,
            Some("Ok"),
            &[
                ("Access-Control-Allow-Origin", "*"),
                ("content-type", "application/json"),
            ],
        )?;
        let gimbal_json = {
            let g = gimbal_arc.lock().unwrap();
            serde_json::to_string(&*g).unwrap()
        };
        write!(response, "{}", gimbal_json)?;
        response.flush()?;
        Ok(())
    })?;

    server.fn_handler("/api/gcode", Method::Get, move |req| {
        let conn = req.connection().unwrap_or("unknown");
        let gcode_str = url::Url::parse(req.uri())?
            .query_pairs()
            .find(|(k, _)| k == "gcode")
            .map(|(_, v)| v.to_string())
            .unwrap_or("".to_string());

        let (code, message, payload) = match GcodeParser::of_str(&gcode_str) {
            Ok(_g) => {
                info!("handling req from connection: {conn}");
                (200, "ok", "{ \"ok\": true }".to_string())
            }
            Err(err) => (
                400,
                "bad param",
                format!("{{ \"error\": \"{err}\" }}").to_string(),
            ),
        };
        let mut response = req.into_response(
            code,
            Some(message),
            &[
                ("Access-Control-Allow-Origin", "*"),
                ("content-type", "application/json"),
            ],
        )?;
        response.write(payload.as_bytes())?;
        response.flush()?;
        Ok(())
    })?;

    Ok(server)
}
