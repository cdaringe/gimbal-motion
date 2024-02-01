use {
    crate::{cmd::Cmd, gcode::GcodeParser, gimbal::Gimbal, server_response::Response},
    embedded_svc::{http::Headers, io::Write, ipv4::IpInfo},
    esp_idf_svc::http::{server::EspHttpServer, Method},
    log::info,
    serde_json,
    std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    },
};

#[derive(serde::Deserialize, serde::Serialize)]
struct PostGcode {
    pub gcode: String,
}

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
        write!(response, "{}", &Response::ok(&*gimbal_arc.lock()?).json()?)?;
        response.flush()?;
        Ok(())
    })?;

    server.fn_handler("/api/gcode", Method::Post, move |mut req| {
        let json_str = {
            let mut buf = [0; 256];
            req.read(&mut buf)?;
            let json_str_raw = String::from_utf8(buf.to_vec())?;
            json_str_raw.trim().to_owned()
        };
        info!("buf: {}", &json_str);
        let body: PostGcode = serde_json::de::from_str(&json_str)?; // serde_json::from_str(json_str)?;
        info!("postgcode: {}", serde_json::to_string(&body)?);
        let (code, message, payload) = match GcodeParser::of_str(&body.gcode) {
            Ok(_g) => (200, "ok", Response::ok(true).json()?),
            Err(err) => (400, "bad param", Response::error(err.to_string()).json()?),
        };
        let mut response = req.into_response(
            code,
            Some(message),
            &[
                ("Access-Control-Allow-Origin", "*"),
                ("content-type", "application/json"),
            ],
        )?;
        info!("pre write");
        response.write(payload.as_bytes())?;
        response.flush()?;
        Ok(())
    })?;

    Ok(server)
}
